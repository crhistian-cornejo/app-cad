//! Main wgpu Renderer - equivalent to Blender's DRW_engine
//!
//! This is the core rendering engine that manages GPU state, pipelines,
//! and frame rendering. Unlike Three.js, this gives us direct GPU control.

use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureFormat};

use crate::camera::Camera;
use crate::error::{ViewportError, ViewportResult};
use crate::render_context::RenderContext;
use crate::scene::Scene;

/// GPU state and rendering context
pub struct Renderer {
    pub instance: Instance,
    pub adapter: Arc<Adapter>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Option<Surface<'static>>,
    pub config: SurfaceConfiguration,
    pub depth_texture: Option<wgpu::Texture>,
    pub depth_view: Option<wgpu::TextureView>,
    pub render_context: Option<RenderContext>,
}

impl Renderer {
    /// Create a new renderer with the given window
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32) -> ViewportResult<Self>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        // Create wgpu instance with best available backend
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance
            .create_surface(window)
            .map_err(|e| ViewportError::SurfaceConfig(e.to_string()))?;

        // Request adapter (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ViewportError::AdapterCreation)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CADHY Viewport Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync, // VSync on
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create render context with pipelines
        let render_context = RenderContext::new(&device, surface_format);

        Ok(Self {
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: Some(surface),
            config,
            depth_texture: Some(depth_texture),
            depth_view: Some(depth_view),
            render_context: Some(render_context),
        })
    }

    /// Create headless renderer (for testing/thumbnails)
    pub async fn new_headless() -> ViewportResult<Self> {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ViewportError::AdapterCreation)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: None,
            config,
            depth_texture: None,
            depth_view: None,
            render_context: None,
        })
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.config);
        }

        // Recreate depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = Some(depth_texture);
        self.depth_view = Some(depth_view);
    }

    /// Create depth texture for z-buffering
    fn create_depth_texture(
        device: &Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
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

    /// Render a frame
    pub fn render(&self, scene: &Scene, camera: &Camera) -> ViewportResult<()> {
        let surface = self
            .surface
            .as_ref()
            .ok_or_else(|| ViewportError::SurfaceConfig("No surface available".to_string()))?;

        let render_context = self
            .render_context
            .as_ref()
            .ok_or_else(|| ViewportError::SurfaceConfig("No render context available".to_string()))?;

        let output = surface
            .get_current_texture()
            .map_err(|e| ViewportError::SurfaceConfig(e.to_string()))?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update camera uniforms
        render_context.update_camera(&self.queue, camera);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(render_context.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: self.depth_view.as_ref().map(|depth_view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render scene using render context
            render_context.render(&mut render_pass, scene, &self.queue);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get surface format
    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    /// Get current size
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get a reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get mutable reference to render context
    pub fn render_context_mut(&mut self) -> Option<&mut RenderContext> {
        self.render_context.as_mut()
    }
}
