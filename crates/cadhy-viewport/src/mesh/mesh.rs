use super::vertex::Vertex;
use wgpu::{Buffer, Device};

/// CPU-side mesh data (for transfer or storage)
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
}

impl MeshData {
    /// Create a cube mesh data
    pub fn cube(size: f32, color: [f32; 4]) -> Self {
        let s = size / 2.0;

        #[rustfmt::skip]
        let vertices = vec![
            // Front face (+Z)
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0]).with_color(color),
            // Back face (-Z)
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0]).with_color(color),
            // Top face (+Y)
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0]).with_color(color),
            // Bottom face (-Y)
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0]).with_color(color),
            // Right face (+X)
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0]).with_color(color),
            // Left face (-X)
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0]).with_color(color),
        ];

        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,   // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Create a UV sphere mesh data
    pub fn sphere(radius: f32, segments: u32, rings: u32, color: [f32; 4]) -> Self {
        use std::f32::consts::PI;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for ring in 0..=rings {
            let phi = PI * ring as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for segment in 0..=segments {
                let theta = 2.0 * PI * segment as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normal = [x / radius, y / radius, z / radius];
                vertices.push(Vertex::new([x, y, z], normal).with_color(color));
            }
        }

        // Generate indices
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                // Two triangles per quad
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Create a cylinder mesh data
    pub fn cylinder(radius: f32, height: f32, segments: u32, color: [f32; 4]) -> Self {
        use std::f32::consts::PI;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let half_height = height / 2.0;

        // Side vertices
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            let normal = [theta.cos(), 0.0, theta.sin()];

            // Bottom vertex
            vertices.push(Vertex::new([x, -half_height, z], normal).with_color(color));
            // Top vertex
            vertices.push(Vertex::new([x, half_height, z], normal).with_color(color));
        }

        // Side indices
        for i in 0..segments {
            let base = i * 2;
            // Triangle 1
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            // Triangle 2
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);
        }

        // Top cap center
        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, half_height, 0.0], [0.0, 1.0, 0.0]).with_color(color));

        // Top cap vertices
        let top_start = vertices.len() as u32;
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            vertices.push(Vertex::new([x, half_height, z], [0.0, 1.0, 0.0]).with_color(color));
        }

        // Top cap indices
        for i in 0..segments {
            indices.push(top_center_idx);
            indices.push(top_start + i);
            indices.push(top_start + i + 1);
        }

        // Bottom cap center
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, -half_height, 0.0], [0.0, -1.0, 0.0]).with_color(color));

        // Bottom cap vertices
        let bottom_start = vertices.len() as u32;
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            vertices.push(Vertex::new([x, -half_height, z], [0.0, -1.0, 0.0]).with_color(color));
        }

        // Bottom cap indices (reversed winding)
        for i in 0..segments {
            indices.push(bottom_center_idx);
            indices.push(bottom_start + i + 1);
            indices.push(bottom_start + i);
        }

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Create a cone mesh data
    pub fn cone(radius: f32, height: f32, segments: u32, color: [f32; 4]) -> Self {
        use std::f32::consts::PI;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let half_height = height / 2.0;

        // Apex
        let apex_idx = 0u32;
        vertices.push(Vertex::new([0.0, half_height, 0.0], [0.0, 1.0, 0.0]).with_color(color));

        // Side vertices (at base)
        let side_start = vertices.len() as u32;
        let slope = radius / height;
        let normal_y = 1.0 / (1.0 + slope * slope).sqrt();
        let normal_xz = slope * normal_y;

        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            let normal = [normal_xz * theta.cos(), normal_y, normal_xz * theta.sin()];
            vertices.push(Vertex::new([x, -half_height, z], normal).with_color(color));
        }

        // Side indices
        for i in 0..segments {
            indices.push(apex_idx);
            indices.push(side_start + i + 1);
            indices.push(side_start + i);
        }

        // Bottom cap center
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, -half_height, 0.0], [0.0, -1.0, 0.0]).with_color(color));

        // Bottom cap vertices
        let bottom_start = vertices.len() as u32;
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            vertices.push(Vertex::new([x, -half_height, z], [0.0, -1.0, 0.0]).with_color(color));
        }

        // Bottom cap indices
        for i in 0..segments {
            indices.push(bottom_center_idx);
            indices.push(bottom_start + i + 1);
            indices.push(bottom_start + i);
        }

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Create a torus mesh data
    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32, color: [f32; 4]) -> Self {
        use std::f32::consts::PI;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=major_segments {
            let u = 2.0 * PI * i as f32 / major_segments as f32;
            let cos_u = u.cos();
            let sin_u = u.sin();

            for j in 0..=minor_segments {
                let v = 2.0 * PI * j as f32 / minor_segments as f32;
                let cos_v = v.cos();
                let sin_v = v.sin();

                let x = (major_radius + minor_radius * cos_v) * cos_u;
                let y = minor_radius * sin_v;
                let z = (major_radius + minor_radius * cos_v) * sin_u;

                let normal = [cos_v * cos_u, sin_v, cos_v * sin_u];
                vertices.push(Vertex::new([x, y, z], normal).with_color(color));
            }
        }

        // Generate indices
        for i in 0..major_segments {
            for j in 0..minor_segments {
                let current = i * (minor_segments + 1) + j;
                let next = (i + 1) * (minor_segments + 1) + j;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Create a plane mesh data
    pub fn plane(width: f32, height: f32, color: [f32; 4]) -> Self {
        let hw = width / 2.0;
        let hh = height / 2.0;

        let vertices = vec![
            Vertex::new([-hw, 0.0, -hh], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([hw, 0.0, -hh], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([hw, 0.0, hh], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([-hw, 0.0, hh], [0.0, 1.0, 0.0]).with_color(color),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self {
            vertices,
            indices: Some(indices),
        }
    }

    /// Upload to GPU
    pub fn upload(&self, device: &Device) -> GpuMesh {
        if let Some(indices) = &self.indices {
            GpuMesh::new_indexed(device, &self.vertices, indices)
        } else {
            GpuMesh::new(device, &self.vertices)
        }
    }
}

/// A mesh stored on the GPU
pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Option<Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl GpuMesh {
    /// Create a new GPU mesh from vertices
    pub fn new(device: &Device, vertices: &[Vertex]) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            index_buffer: None,
            vertex_count: vertices.len() as u32,
            index_count: 0,
        }
    }

    /// Create a new GPU mesh with indices
    pub fn new_indexed(device: &Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer: Some(index_buffer),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        }
    }

    /// Check if mesh uses indices
    pub fn is_indexed(&self) -> bool {
        self.index_buffer.is_some()
    }

    /// Create a simple cube mesh (for testing) with default gray color
    pub fn cube(device: &Device, size: f32) -> Self {
        Self::cube_colored(device, size, [0.7, 0.7, 0.7, 1.0])
    }

    /// Create a cube mesh with a specific color
    pub fn cube_colored(device: &Device, size: f32, color: [f32; 4]) -> Self {
        let s = size / 2.0;
        
        #[rustfmt::skip]
        let vertices = vec![
            // Front face (+Z)
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0]).with_color(color),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0]).with_color(color),
            // Back face (-Z)
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0]).with_color(color),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0]).with_color(color),
            // Top face (+Y)
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0]).with_color(color),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0]).with_color(color),
            // Bottom face (-Y)
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0]).with_color(color),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0]).with_color(color),
            // Right face (+X)
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0]).with_color(color),
            // Left face (-X)
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0]).with_color(color),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0]).with_color(color),
        ];

        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,   // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self::new_indexed(device, &vertices, &indices)
    }
    
    /// Create a colored cube with different colors per face (for testing)
    pub fn cube_multicolor(device: &Device, size: f32) -> Self {
        let s = size / 2.0;
        
        // Colors for each face
        let red = [0.9, 0.2, 0.2, 1.0];
        let green = [0.2, 0.9, 0.2, 1.0];
        let blue = [0.2, 0.2, 0.9, 1.0];
        let yellow = [0.9, 0.9, 0.2, 1.0];
        let cyan = [0.2, 0.9, 0.9, 1.0];
        let magenta = [0.9, 0.2, 0.9, 1.0];
        
        #[rustfmt::skip]
        let vertices = vec![
            // Front face (+Z) - Blue
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0]).with_color(blue),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0]).with_color(blue),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0]).with_color(blue),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0]).with_color(blue),
            // Back face (-Z) - Yellow
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0]).with_color(yellow),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0]).with_color(yellow),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0]).with_color(yellow),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0]).with_color(yellow),
            // Top face (+Y) - Green
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0]).with_color(green),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0]).with_color(green),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0]).with_color(green),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0]).with_color(green),
            // Bottom face (-Y) - Magenta
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0]).with_color(magenta),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0]).with_color(magenta),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0]).with_color(magenta),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0]).with_color(magenta),
            // Right face (+X) - Red
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0]).with_color(red),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0]).with_color(red),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0]).with_color(red),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0]).with_color(red),
            // Left face (-X) - Cyan
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0]).with_color(cyan),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0]).with_color(cyan),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0]).with_color(cyan),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0]).with_color(cyan),
        ];

        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,   // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self::new_indexed(device, &vertices, &indices)
    }

    /// Create a grid mesh (for viewport floor)
    pub fn grid(device: &Device, size: f32, divisions: u32) -> Self {
        let mut vertices = Vec::new();
        let step = size / divisions as f32;
        let half = size / 2.0;
        let color = [0.3, 0.3, 0.3, 1.0];

        for i in 0..=divisions {
            let pos = -half + i as f32 * step;
            
            // Lines along X
            vertices.push(Vertex::new([-half, 0.0, pos], [0.0, 1.0, 0.0]).with_color(color));
            vertices.push(Vertex::new([half, 0.0, pos], [0.0, 1.0, 0.0]).with_color(color));
            
            // Lines along Z
            vertices.push(Vertex::new([pos, 0.0, -half], [0.0, 1.0, 0.0]).with_color(color));
            vertices.push(Vertex::new([pos, 0.0, half], [0.0, 1.0, 0.0]).with_color(color));
        }

        Self::new(device, &vertices)
    }
}