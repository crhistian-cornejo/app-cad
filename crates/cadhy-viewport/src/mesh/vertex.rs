use bytemuck::{Pod, Zeroable};

/// Vertex format for 3D meshes
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x3,  // normal
        2 => Float32x4,  // color
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            normal,
            color: [0.8, 0.8, 0.8, 1.0], // Default gray
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}