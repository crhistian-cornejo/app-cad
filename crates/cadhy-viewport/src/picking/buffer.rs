use wgpu::{Device, Texture, TextureFormat, TextureView};

use crate::error::ViewportResult;

/// Picking buffer for GPU-based object selection
pub struct PickingBuffer {
    pub texture: Texture,
    pub view: TextureView,
    pub width: u32,
    pub height: u32,
    /// CPU-readable buffer for picking results
    pub read_buffer: wgpu::Buffer,
}

impl PickingBuffer {
    /// Create a new picking buffer
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Picking Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Buffer for reading back a single pixel
        let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Picking Read Buffer"),
            size: 256, // Aligned to 256 bytes as required by wgpu
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            texture,
            view,
            width,
            height,
            read_buffer,
        }
    }

    /// Resize the picking buffer
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        *self = Self::new(device, width, height);
    }

    /// Read pick color at the given screen coordinates
    pub async fn read_pixel_async(
        &self,
        device: &Device,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
    ) -> ViewportResult<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return Ok([0, 0, 0, 0]);
        }

        // Create command to copy pixel to read buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Pick Read Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.read_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(256), // Aligned
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read result
        let buffer_slice = self.read_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        
        device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().map_err(|e| crate::error::ViewportError::BufferCreation(e.to_string()))?;

        let data = buffer_slice.get_mapped_range();
        let pixel = [data[0], data[1], data[2], data[3]];
        
        drop(data);
        self.read_buffer.unmap();

        Ok(pixel)
    }

    /// Synchronous pixel read (blocks until complete)
    pub fn read_pixel_sync(
        &self,
        device: &Device,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
    ) -> ViewportResult<[u8; 4]> {
        // Use pollster to block on async
        pollster::block_on(self.read_pixel_async(device, queue, x, y))
    }
}