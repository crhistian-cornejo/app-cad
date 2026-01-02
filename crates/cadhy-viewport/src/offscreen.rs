//! Offscreen Renderer - renders to a texture for screenshot/preview
//!
//! This module provides offscreen rendering capabilities for:
//! - Thumbnails and previews
//! - Screenshot export
//! - Streaming frames to webview (fallback mode)

use wgpu::{Device, Queue, TextureFormat};

use crate::camera::Camera;
use crate::error::{ViewportError, ViewportResult};
use crate::render_context::RenderContext;
use crate::scene::Scene;

/// Offscreen renderer for rendering to a buffer
pub struct OffscreenRenderer {
    pub device: Device,
    pub queue: Queue,
    pub render_context: RenderContext,
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    format: TextureFormat,
}

impl OffscreenRenderer {
    /// Create a new offscreen renderer
    pub async fn new(width: u32, height: u32) -> ViewportResult<Self> {
        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ViewportError::AdapterCreation)?;

        // Request device with features needed for wireframe rendering
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CADHY Offscreen Device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        let format = TextureFormat::Rgba8UnormSrgb;

        // Create output texture
        let (output_texture, output_view) = Self::create_output_texture(&device, width, height, format);

        // Create depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create output buffer for reading pixels
        // wgpu requires bytes_per_row to be aligned to 256
        let bytes_per_row = width * 4;
        let padded_bytes_per_row = (bytes_per_row + 255) & !255;
        let buffer_size = (padded_bytes_per_row * height) as u64;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offscreen Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create render context
        let render_context = RenderContext::new(&device, format);

        Ok(Self {
            device,
            queue,
            render_context,
            output_texture,
            output_view,
            depth_texture,
            depth_view,
            output_buffer,
            width,
            height,
            format,
        })
    }

    fn create_output_texture(
        device: &Device,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_depth_texture(
        device: &Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Resize the offscreen buffer
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 || (width == self.width && height == self.height) {
            return;
        }

        self.width = width;
        self.height = height;

        // Recreate textures
        let (output_texture, output_view) =
            Self::create_output_texture(&self.device, width, height, self.format);
        self.output_texture = output_texture;
        self.output_view = output_view;

        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        // Recreate output buffer with proper padding
        let bytes_per_row = width * 4;
        let padded_bytes_per_row = (bytes_per_row + 255) & !255;
        let buffer_size = (padded_bytes_per_row * height) as u64;
        self.output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offscreen Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    }

    /// Render a frame to the offscreen buffer
    pub fn render(&self, scene: &Scene, camera: &Camera) -> ViewportResult<()> {
        // Update camera uniforms
        self.render_context.update_camera(&self.queue, camera);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen Render Encoder"),
            });

        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
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

            // Render scene using render context
            self.render_context.render(&mut render_pass, scene, &self.queue);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Render and get the frame as RGBA bytes (synchronous version)
    /// This is the preferred method when calling from a blocking context
    pub fn render_to_bytes_sync(&self, scene: &Scene, camera: &Camera) -> ViewportResult<Vec<u8>> {
        // Render the frame
        self.render(scene, camera)?;

        // Copy texture to buffer - combine with previous submit to reduce sync points
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });

        let bytes_per_row = self.width * 4;
        // wgpu requires bytes_per_row to be aligned to 256
        let padded_bytes_per_row = (bytes_per_row + 255) & !255;

        #[allow(deprecated)]
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data - use polling with small sleeps to reduce CPU usage
        let buffer_slice = self.output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // Poll with maintain to allow other work while waiting
        self.device.poll(wgpu::Maintain::Wait);

        rx.recv()
            .map_err(|e| ViewportError::SurfaceConfig(e.to_string()))?
            .map_err(|e| ViewportError::SurfaceConfig(format!("{:?}", e)))?;

        let data = buffer_slice.get_mapped_range();

        // Pre-allocate with exact capacity and use unsafe for speed
        let total_bytes = (self.width * self.height * 4) as usize;
        let mut result = Vec::with_capacity(total_bytes);
        
        // Fast path: no padding needed
        if padded_bytes_per_row == bytes_per_row {
            result.extend_from_slice(&data[..total_bytes]);
        } else {
            // Remove padding - use chunks for better cache locality
            for row in 0..self.height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + (self.width * 4) as usize;
                result.extend_from_slice(&data[start..end]);
            }
        }

        drop(data);
        self.output_buffer.unmap();

        Ok(result)
    }

    /// Render and get the frame as RGBA bytes (async version)
    /// Note: This internally uses blocking operations, prefer render_to_bytes_sync
    /// when calling from tokio::task::spawn_blocking
    pub async fn render_to_bytes(&self, scene: &Scene, camera: &Camera) -> ViewportResult<Vec<u8>> {
        self.render_to_bytes_sync(scene, camera)
    }

    /// Get current size
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get a reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
