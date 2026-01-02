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
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration, TextureFormat};

use crate::camera::Camera;
use crate::error::{ViewportError, ViewportResult};
use crate::render_context::RenderContext;
use crate::scene::Scene;

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
        // Create wgpu instance with best available backend
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window handle
        let surface = instance
            .create_surface(window)
            .map_err(|e| ViewportError::SurfaceConfig(e.to_string()))?;

        // Request high-performance GPU adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ViewportError::AdapterCreation)?;

        // Log adapter info
        let info = adapter.get_info();
        tracing::info!(
            "GPU: {} ({:?})",
            info.name,
            info.backend
        );

        // Request device with features for wireframe rendering
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

        // Configure surface for optimal performance
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Use AutoVsync for smooth 60 FPS with minimal latency
        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::AutoVsync) {
            wgpu::PresentMode::AutoVsync
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            surface_caps.present_modes[0]
        };

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create render context with all pipelines
        let render_context = RenderContext::new(&device, surface_format);

        let device = Arc::new(device);
        let queue = Arc::new(queue);

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
}
