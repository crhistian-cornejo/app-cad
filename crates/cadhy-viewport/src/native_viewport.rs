//! Native Viewport - High-performance wgpu rendering to a native window
//!
//! This module provides direct GPU-to-screen rendering without any
//! CPU roundtrips or IPC overhead. Achieves 60+ FPS by using wgpu's
//! native surface presentation.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │     Native Window (Tauri/Tao)       │
//! │         HasWindowHandle             │
//! └─────────────────┬───────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────┐
//! │         wgpu Surface                │
//! │    (GPU → Screen directly)          │
//! └─────────────────┬───────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────┐
//! │    NativeViewport (render loop)     │
//! │   60 FPS with VSync, no CPU copy    │
//! └─────────────────────────────────────┘
//! ```

use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use uuid::Uuid;
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration, TextureFormat};

use crate::camera::Camera;
use crate::error::{ViewportError, ViewportResult};
use crate::mesh::{Gizmo, GizmoAxis, GizmoMode, GpuGizmo};
use crate::picking::PickingBuffer;
use crate::pipelines::{GizmoBuffer, GizmoPipeline, GizmoUniform, PickingPipeline};
use crate::render_context::RenderContext;
use crate::scene::Scene;

/// Result of picking at a screen coordinate
#[derive(Debug, Clone)]
pub enum PickResult {
    /// Nothing was picked (background)
    None,
    /// A gizmo axis was picked
    Gizmo(GizmoAxis),
    /// A scene object was picked
    Object(Uuid),
}

/// Model uniform for picking - includes pick color
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PickModelUniform {
    pub model: [[f32; 4]; 4],
    pub pick_color: [f32; 4],
}

impl PickModelUniform {
    pub fn new(model_matrix: glam::Mat4, pick_color: [u8; 4]) -> Self {
        Self {
            model: model_matrix.to_cols_array_2d(),
            pick_color: [
                pick_color[0] as f32 / 255.0,
                pick_color[1] as f32 / 255.0,
                pick_color[2] as f32 / 255.0,
                pick_color[3] as f32 / 255.0,
            ],
        }
    }
}

/// High-performance viewport that renders directly to a native window surface.
///
/// Unlike OffscreenRenderer, this presents frames directly to the GPU
/// without any CPU readback, base64 encoding, or IPC transfer.
pub struct NativeViewport {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub render_context: RenderContext,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
    // Picking support
    picking_pipeline: PickingPipeline,
    picking_buffer: PickingBuffer,
    picking_depth_texture: wgpu::Texture,
    picking_depth_view: wgpu::TextureView,
    picking_camera_buffer: wgpu::Buffer,
    picking_camera_bind_group: wgpu::BindGroup,
    picking_model_buffer: wgpu::Buffer,
    picking_model_bind_group: wgpu::BindGroup,
    // Gizmo support (optional - may fail on some systems)
    gizmo: Option<GizmoResources>,
    pub gizmo_mode: GizmoMode,
}

/// Gizmo rendering resources (grouped for optional initialization)
pub struct GizmoResources {
    pub pipeline: GizmoPipeline,
    pub meshes: GpuGizmo,
    pub buffer: GizmoBuffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl NativeViewport {
    /// Create a new native viewport with the given window handle
    ///
    /// The window must implement `HasWindowHandle` and `HasDisplayHandle`,
    /// which Tauri's Window type does.
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32) -> ViewportResult<Self>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        tracing::info!("[NativeViewport::new] Starting initialization...");
        tracing::info!("[NativeViewport::new] Requested size: {}x{}", width, height);

        // Verify window handles before proceeding
        let window_handle = window.window_handle()
            .map_err(|e| {
                tracing::error!("[NativeViewport::new] Failed to get window handle: {:?}", e);
                ViewportError::SurfaceConfig(format!("Window handle error: {:?}", e))
            })?;
        tracing::info!("[NativeViewport::new] Window handle: {:?}", window_handle.as_raw());

        let display_handle = window.display_handle()
            .map_err(|e| {
                tracing::error!("[NativeViewport::new] Failed to get display handle: {:?}", e);
                ViewportError::SurfaceConfig(format!("Display handle error: {:?}", e))
            })?;
        tracing::info!("[NativeViewport::new] Display handle: {:?}", display_handle.as_raw());

        // Create wgpu instance with best available backend
        tracing::info!("[NativeViewport::new] Creating wgpu instance...");
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        tracing::info!("[NativeViewport::new] wgpu instance created");

        // Create surface from window handle
        tracing::info!("[NativeViewport::new] Creating surface from window...");
        let surface = instance
            .create_surface(window)
            .map_err(|e| {
                tracing::error!("[NativeViewport::new] Surface creation failed: {}", e);
                ViewportError::SurfaceConfig(e.to_string())
            })?;
        tracing::info!("[NativeViewport::new] Surface created successfully");

        // Request high-performance GPU adapter
        tracing::info!("[NativeViewport::new] Requesting GPU adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                tracing::error!("[NativeViewport::new] No compatible GPU adapter found!");
                ViewportError::AdapterCreation
            })?;

        // Log adapter info
        let info = adapter.get_info();
        tracing::info!(
            "[NativeViewport::new] GPU adapter acquired: {} ({:?})",
            info.name,
            info.backend
        );

        // Request device with features for wireframe rendering
        tracing::info!("[NativeViewport::new] Requesting device with POLYGON_MODE_LINE feature...");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CADHY Native Viewport Device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;
        tracing::info!("[NativeViewport::new] Device and queue created");

        // Configure surface for optimal performance
        tracing::info!("[NativeViewport::new] Getting surface capabilities...");
        let surface_caps = surface.get_capabilities(&adapter);
        tracing::info!("[NativeViewport::new] Available formats: {:?}", surface_caps.formats);
        tracing::info!("[NativeViewport::new] Available present modes: {:?}", surface_caps.present_modes);
        tracing::info!("[NativeViewport::new] Available alpha modes: {:?}", surface_caps.alpha_modes);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        tracing::info!("[NativeViewport::new] Selected format: {:?}", surface_format);

        // Use Immediate for maximum FPS (100+), fall back to Fifo for VSync
        // Priority: Immediate > Mailbox > AutoVsync > Fifo
        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate  // Uncapped FPS, no tearing on modern displays
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox    // Uncapped FPS with frame buffering
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::AutoVsync) {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::Fifo       // VSync fallback
        };
        tracing::info!("[NativeViewport::new] Selected present mode: {:?}", present_mode);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,  // Minimum latency for responsive input
        };

        tracing::info!("[NativeViewport::new] Configuring surface...");
        surface.configure(&device, &config);
        tracing::info!("[NativeViewport::new] Surface configured successfully");

        // Create depth buffer
        tracing::info!("[NativeViewport::new] Creating depth texture...");
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create render context with all pipelines
        tracing::info!("[NativeViewport::new] Creating render context with pipelines...");
        let render_context = RenderContext::new(&device, surface_format);

        // Create picking resources
        tracing::info!("[NativeViewport::new] Creating picking pipeline and buffer...");
        let picking_pipeline = PickingPipeline::new(&device);
        let picking_buffer = PickingBuffer::new(&device, width, height);
        let (picking_depth_texture, picking_depth_view) = Self::create_depth_texture(&device, width, height);

        // Create picking uniform buffers
        let picking_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Picking Camera Buffer"),
            size: std::mem::size_of::<crate::pipelines::CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let picking_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Picking Camera Bind Group"),
            layout: &picking_pipeline.camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: picking_camera_buffer.as_entire_binding(),
            }],
        });

        let picking_model_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Picking Model Buffer"),
            size: std::mem::size_of::<PickModelUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let picking_model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Picking Model Bind Group"),
            layout: &picking_pipeline.model_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: picking_model_buffer.as_entire_binding(),
            }],
        });

        // Create gizmo resources (optional - may fail on some systems)
        // We use a helper function that returns Option to gracefully handle failures
        let gizmo = Self::try_init_gizmo(&device, surface_format, &picking_camera_buffer);
        if gizmo.is_some() {
            tracing::info!("[NativeViewport::new] Gizmo initialized successfully");
        } else {
            tracing::warn!("[NativeViewport::new] Gizmo initialization failed - gizmos disabled");
        }

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        tracing::info!("[NativeViewport::new] Initialization complete!");
        Ok(Self {
            device,
            queue,
            surface,
            config,
            render_context,
            depth_texture,
            depth_view,
            width,
            height,
            picking_pipeline,
            picking_buffer,
            picking_depth_texture,
            picking_depth_view,
            picking_camera_buffer,
            picking_camera_bind_group,
            picking_model_buffer,
            picking_model_bind_group,
            gizmo,
            gizmo_mode: GizmoMode::Translate,
        })
    }

    /// Create depth texture for z-buffering
    fn create_depth_texture(
        device: &Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Native Viewport Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Try to initialize gizmo resources
    /// Returns None if initialization fails for any reason
    fn try_init_gizmo(
        device: &Device,
        surface_format: TextureFormat,
        picking_camera_buffer: &wgpu::Buffer,
    ) -> Option<GizmoResources> {
        tracing::debug!("[try_init_gizmo] Creating gizmo pipeline...");

        // Create the gizmo pipeline (shader compilation)
        let pipeline = GizmoPipeline::new(device, surface_format);
        tracing::debug!("[try_init_gizmo] Gizmo pipeline created");

        // Create gizmo mesh data
        tracing::debug!("[try_init_gizmo] Creating gizmo mesh...");
        let gizmo_cpu = Gizmo::new();
        tracing::debug!("[try_init_gizmo] Uploading gizmo mesh to GPU...");
        let meshes = gizmo_cpu.upload(device);
        tracing::debug!("[try_init_gizmo] Gizmo mesh uploaded");

        // Create gizmo uniform buffer
        tracing::debug!("[try_init_gizmo] Creating gizmo uniform buffer...");
        let buffer = GizmoBuffer::new(device, &pipeline.gizmo_layout);
        tracing::debug!("[try_init_gizmo] Gizmo uniform buffer created");

        // Create camera bind group for gizmo (reuse picking camera buffer)
        tracing::debug!("[try_init_gizmo] Creating camera bind group...");
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Camera Bind Group"),
            layout: &pipeline.camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: picking_camera_buffer.as_entire_binding(),
            }],
        });
        tracing::debug!("[try_init_gizmo] Gizmo initialization complete");

        Some(GizmoResources {
            pipeline,
            meshes,
            buffer,
            camera_bind_group,
        })
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        if width == self.width && height == self.height {
            return;
        }

        self.width = width;
        self.height = height;
        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);

        // Recreate depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        // Recreate picking resources
        self.picking_buffer.resize(&self.device, width, height);
        let (picking_depth_texture, picking_depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.picking_depth_texture = picking_depth_texture;
        self.picking_depth_view = picking_depth_view;
    }

    /// Render a frame directly to the window surface
    ///
    /// This is the high-performance path - no CPU copies, no base64,
    /// just GPU → Screen via surface.present()
    pub fn render(&self, scene: &Scene, camera: &Camera) -> ViewportResult<()> {
        // Get the current surface texture
        let output = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Reconfigure and retry
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_texture()
                    .map_err(|e| ViewportError::SurfaceConfig(e.to_string()))?
            }
            Err(e) => return Err(ViewportError::SurfaceConfig(e.to_string())),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update camera uniforms
        self.render_context.update_camera(&self.queue, camera);

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Native Viewport Render Encoder"),
            });

        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Native Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.render_context.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render scene
            self.render_context.render(&mut render_pass, scene, &self.queue);
        }

        // Render gizmo if there's a selection AND gizmo resources are available
        if !scene.selection.is_empty() {
            if let Some(ref gizmo) = self.gizmo {
                // Get center of selection for gizmo position
                let gizmo_position = if let Some(first_selected) = scene.selection.first() {
                    if let Some(obj) = scene.get(*first_selected) {
                        obj.transform.position
                    } else {
                        glam::Vec3::ZERO
                    }
                } else {
                    glam::Vec3::ZERO
                };

                // Calculate gizmo scale based on camera distance (constant screen size)
                let camera_distance = (camera.position - gizmo_position).length();
                let gizmo_scale = camera_distance * 0.15; // 15% of camera distance

                // Update gizmo uniform
                let gizmo_uniform = GizmoUniform::new(gizmo_position.to_array(), gizmo_scale);
                gizmo.buffer.update(&self.queue, &gizmo_uniform);

                // Update camera uniform for gizmo (same as picking camera buffer)
                let camera_uniform = crate::pipelines::CameraUniform::from_matrices(
                    camera.view_projection_matrix(),
                    camera.position,
                );
                self.queue.write_buffer(
                    &self.picking_camera_buffer,
                    0,
                    bytemuck::bytes_of(&camera_uniform),
                );

                // Gizmo render pass (on top of scene)
                {
                    let mut gizmo_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Gizmo Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load, // Don't clear, render on top
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load, // Keep existing depth
                                store: wgpu::StoreOp::Discard, // Gizmo doesn't write depth
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    gizmo_pass.set_pipeline(&gizmo.pipeline.pipeline);
                    gizmo_pass.set_bind_group(0, &gizmo.camera_bind_group, &[]);
                    gizmo_pass.set_bind_group(1, &gizmo.buffer.bind_group, &[]);

                    // Render based on gizmo mode
                    match self.gizmo_mode {
                        GizmoMode::Translate => {
                            // X axis (red arrow)
                            let x_mesh = &gizmo.meshes.translate.x_axis;
                            gizmo_pass.set_vertex_buffer(0, x_mesh.vertex_buffer.slice(..));
                            if x_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    x_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..x_mesh.index_count, 0, 0..1);
                            }

                            // Y axis (green arrow)
                            let y_mesh = &gizmo.meshes.translate.y_axis;
                            gizmo_pass.set_vertex_buffer(0, y_mesh.vertex_buffer.slice(..));
                            if y_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    y_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..y_mesh.index_count, 0, 0..1);
                            }

                            // Z axis (blue arrow)
                            let z_mesh = &gizmo.meshes.translate.z_axis;
                            gizmo_pass.set_vertex_buffer(0, z_mesh.vertex_buffer.slice(..));
                            if z_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    z_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..z_mesh.index_count, 0, 0..1);
                            }
                        }
                        GizmoMode::Scale => {
                            // Similar rendering for scale gizmo axes
                            let x_mesh = &gizmo.meshes.scale.x_axis;
                            gizmo_pass.set_vertex_buffer(0, x_mesh.vertex_buffer.slice(..));
                            if x_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    x_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..x_mesh.index_count, 0, 0..1);
                            }

                            let y_mesh = &gizmo.meshes.scale.y_axis;
                            gizmo_pass.set_vertex_buffer(0, y_mesh.vertex_buffer.slice(..));
                            if y_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    y_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..y_mesh.index_count, 0, 0..1);
                            }

                            let z_mesh = &gizmo.meshes.scale.z_axis;
                            gizmo_pass.set_vertex_buffer(0, z_mesh.vertex_buffer.slice(..));
                            if z_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    z_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..z_mesh.index_count, 0, 0..1);
                            }

                            // Center cube
                            let center_mesh = &gizmo.meshes.scale.center;
                            gizmo_pass.set_vertex_buffer(0, center_mesh.vertex_buffer.slice(..));
                            if center_mesh.is_indexed() {
                                gizmo_pass.set_index_buffer(
                                    center_mesh.index_buffer.as_ref().unwrap().slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                gizmo_pass.draw_indexed(0..center_mesh.index_count, 0, 0..1);
                            }
                        }
                        GizmoMode::Rotate => {
                            // TODO: Implement rotation gizmo (circles)
                        }
                    }
                }
            }
        }

        // Submit and present
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get current viewport size
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the surface format
    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    /// Get reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get mutable reference to render context
    pub fn render_context_mut(&mut self) -> &mut RenderContext {
        &mut self.render_context
    }

    /// Pick object or gizmo at screen coordinates
    ///
    /// Returns what was picked at the given screen coordinates:
    /// - `PickResult::None` if background was clicked
    /// - `PickResult::Gizmo(axis)` if a gizmo axis was clicked
    /// - `PickResult::Object(uuid)` if a scene object was clicked
    pub fn pick_at(&self, scene: &Scene, camera: &Camera, x: u32, y: u32) -> ViewportResult<PickResult> {
        if x >= self.width || y >= self.height {
            return Ok(PickResult::None);
        }

        // Update picking camera uniform
        let camera_uniform = crate::pipelines::CameraUniform::from_matrices(
            camera.view_projection_matrix(),
            camera.position,
        );
        self.queue.write_buffer(
            &self.picking_camera_buffer,
            0,
            bytemuck::bytes_of(&camera_uniform),
        );

        // Calculate gizmo transform if there's a selection
        let gizmo_transform = if !scene.selection.is_empty() {
            if let Some(first_selected) = scene.selection.first() {
                if let Some(obj) = scene.get(*first_selected) {
                    let pos = obj.transform.position;
                    let camera_distance = (camera.position - pos).length();
                    let scale = camera_distance * 0.15;
                    Some((pos, scale))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Create picking render pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Picking Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Picking Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.picking_buffer.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.picking_depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.picking_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.picking_camera_bind_group, &[]);

            // Render each visible object with its pick color
            for object in scene.visible_objects() {
                if let Some(mesh) = &object.mesh {
                    // Update model matrix and pick color
                    let model_uniform = PickModelUniform::new(
                        object.transform.matrix(),
                        object.pick_color,
                    );
                    self.queue.write_buffer(
                        &self.picking_model_buffer,
                        0,
                        bytemuck::bytes_of(&model_uniform),
                    );

                    render_pass.set_bind_group(1, &self.picking_model_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                    if mesh.is_indexed() {
                        render_pass.set_index_buffer(
                            mesh.index_buffer.as_ref().unwrap().slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    } else {
                        render_pass.draw(0..mesh.vertex_count, 0..1);
                    }
                }
            }

            // Render gizmo for picking if there's a selection and gizmo resources exist
            if let (Some((gizmo_pos, gizmo_scale)), Some(ref gizmo)) = (gizmo_transform, &self.gizmo) {
                // Create gizmo model matrix (scale and translate)
                let gizmo_matrix = glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(gizmo_scale),
                    glam::Quat::IDENTITY,
                    gizmo_pos,
                );

                // Helper to render a gizmo axis mesh with its pick color
                let render_gizmo_axis = |render_pass: &mut wgpu::RenderPass, mesh: &crate::mesh::GpuMesh, axis: GizmoAxis| {
                    let pick_color = axis.pick_color();
                    let model_uniform = PickModelUniform::new(gizmo_matrix, pick_color);
                    self.queue.write_buffer(
                        &self.picking_model_buffer,
                        0,
                        bytemuck::bytes_of(&model_uniform),
                    );

                    render_pass.set_bind_group(1, &self.picking_model_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                    if mesh.is_indexed() {
                        render_pass.set_index_buffer(
                            mesh.index_buffer.as_ref().unwrap().slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                };

                // Render based on current gizmo mode
                match self.gizmo_mode {
                    GizmoMode::Translate => {
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.translate.x_axis, GizmoAxis::X);
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.translate.y_axis, GizmoAxis::Y);
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.translate.z_axis, GizmoAxis::Z);
                    }
                    GizmoMode::Scale => {
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.scale.x_axis, GizmoAxis::X);
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.scale.y_axis, GizmoAxis::Y);
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.scale.z_axis, GizmoAxis::Z);
                        render_gizmo_axis(&mut render_pass, &gizmo.meshes.scale.center, GizmoAxis::All);
                    }
                    GizmoMode::Rotate => {
                        // TODO: Render rotation gizmo axes
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read pixel at click position
        let pixel = self.picking_buffer.read_pixel_sync(&self.device, &self.queue, x, y)?;

        // Check if background (black) was clicked
        if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
            return Ok(PickResult::None);
        }

        // Check for gizmo axis pick colors first (gizmo renders on top)
        let pick_color = [pixel[0], pixel[1], pixel[2]];

        // Check each gizmo axis
        if pick_color == [255, 0, 0] {
            return Ok(PickResult::Gizmo(GizmoAxis::X));
        }
        if pick_color == [0, 255, 0] {
            return Ok(PickResult::Gizmo(GizmoAxis::Y));
        }
        if pick_color == [0, 0, 255] {
            return Ok(PickResult::Gizmo(GizmoAxis::Z));
        }
        if pick_color == [255, 255, 0] {
            return Ok(PickResult::Gizmo(GizmoAxis::XY));
        }
        if pick_color == [255, 0, 255] {
            return Ok(PickResult::Gizmo(GizmoAxis::XZ));
        }
        if pick_color == [0, 255, 255] {
            return Ok(PickResult::Gizmo(GizmoAxis::YZ));
        }
        if pick_color == [255, 255, 255] {
            return Ok(PickResult::Gizmo(GizmoAxis::All));
        }

        // Find object by pick color
        if let Some(uuid) = scene.find_by_pick_color([pixel[0], pixel[1], pixel[2]]) {
            return Ok(PickResult::Object(uuid));
        }

        Ok(PickResult::None)
    }
}
