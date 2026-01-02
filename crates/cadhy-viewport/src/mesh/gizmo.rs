//! Gizmo mesh data for transform handles
//!
//! Provides mesh generation for translation, rotation, and scale gizmos.

use super::vertex::Vertex;
use super::mesh::{GpuMesh, MeshData};
use wgpu::Device;

/// Gizmo transform mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Gizmo axis identifier (for picking and highlighting)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    XY,  // XY plane
    XZ,  // XZ plane
    YZ,  // YZ plane
    All, // Center/all axes (uniform scale)
    None,
}

impl GizmoAxis {
    /// Get the color for this axis
    pub fn color(&self) -> [f32; 4] {
        match self {
            GizmoAxis::X => [1.0, 0.2, 0.2, 1.0],   // Red
            GizmoAxis::Y => [0.2, 1.0, 0.2, 1.0],   // Green
            GizmoAxis::Z => [0.2, 0.2, 1.0, 1.0],   // Blue
            GizmoAxis::XY => [1.0, 1.0, 0.2, 0.6],  // Yellow (semi-transparent)
            GizmoAxis::XZ => [1.0, 0.2, 1.0, 0.6],  // Magenta (semi-transparent)
            GizmoAxis::YZ => [0.2, 1.0, 1.0, 0.6],  // Cyan (semi-transparent)
            GizmoAxis::All => [1.0, 1.0, 1.0, 1.0], // White
            GizmoAxis::None => [0.5, 0.5, 0.5, 0.5],
        }
    }

    /// Get highlight color (brighter version)
    pub fn highlight_color(&self) -> [f32; 4] {
        match self {
            GizmoAxis::X => [1.0, 0.5, 0.5, 1.0],
            GizmoAxis::Y => [0.5, 1.0, 0.5, 1.0],
            GizmoAxis::Z => [0.5, 0.5, 1.0, 1.0],
            GizmoAxis::XY => [1.0, 1.0, 0.5, 0.8],
            GizmoAxis::XZ => [1.0, 0.5, 1.0, 0.8],
            GizmoAxis::YZ => [0.5, 1.0, 1.0, 0.8],
            GizmoAxis::All => [1.0, 1.0, 1.0, 1.0],
            GizmoAxis::None => [0.5, 0.5, 0.5, 0.5],
        }
    }

    /// Get pick color (unique color for GPU picking)
    pub fn pick_color(&self) -> [u8; 4] {
        match self {
            GizmoAxis::X => [255, 0, 0, 255],
            GizmoAxis::Y => [0, 255, 0, 255],
            GizmoAxis::Z => [0, 0, 255, 255],
            GizmoAxis::XY => [255, 255, 0, 255],
            GizmoAxis::XZ => [255, 0, 255, 255],
            GizmoAxis::YZ => [0, 255, 255, 255],
            GizmoAxis::All => [255, 255, 255, 255],
            GizmoAxis::None => [0, 0, 0, 255],
        }
    }
}

/// Translation gizmo mesh data
/// Creates arrows pointing along each axis
pub struct TranslateGizmo {
    pub x_axis: MeshData,
    pub y_axis: MeshData,
    pub z_axis: MeshData,
}

impl TranslateGizmo {
    /// Create translation gizmo with given arrow length and head size
    pub fn new(length: f32, head_length: f32, head_radius: f32, shaft_radius: f32) -> Self {
        Self {
            x_axis: Self::create_arrow(length, head_length, head_radius, shaft_radius, GizmoAxis::X),
            y_axis: Self::create_arrow(length, head_length, head_radius, shaft_radius, GizmoAxis::Y),
            z_axis: Self::create_arrow(length, head_length, head_radius, shaft_radius, GizmoAxis::Z),
        }
    }

    /// Create default-sized translation gizmo
    pub fn default_size() -> Self {
        Self::new(1.0, 0.2, 0.08, 0.02)
    }

    /// Create an arrow along the specified axis
    fn create_arrow(length: f32, head_length: f32, head_radius: f32, shaft_radius: f32, axis: GizmoAxis) -> MeshData {
        use std::f32::consts::PI;

        let color = axis.color();
        let segments = 12u32;
        let shaft_length = length - head_length;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Create shaft (cylinder from origin along +axis)
        // Shaft vertices
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Get position based on axis
            let (start, end, normal) = match axis {
                GizmoAxis::X => (
                    [0.0, shaft_radius * cos_t, shaft_radius * sin_t],
                    [shaft_length, shaft_radius * cos_t, shaft_radius * sin_t],
                    [0.0, cos_t, sin_t],
                ),
                GizmoAxis::Y => (
                    [shaft_radius * cos_t, 0.0, shaft_radius * sin_t],
                    [shaft_radius * cos_t, shaft_length, shaft_radius * sin_t],
                    [cos_t, 0.0, sin_t],
                ),
                GizmoAxis::Z => (
                    [shaft_radius * cos_t, shaft_radius * sin_t, 0.0],
                    [shaft_radius * cos_t, shaft_radius * sin_t, shaft_length],
                    [cos_t, sin_t, 0.0],
                ),
                _ => continue,
            };

            vertices.push(Vertex::new(start, normal).with_color(color));
            vertices.push(Vertex::new(end, normal).with_color(color));
        }

        // Shaft indices
        for i in 0..segments {
            let base = i * 2;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);
        }

        // Create cone head
        let cone_base = vertices.len() as u32;

        // Apex of cone
        let apex = match axis {
            GizmoAxis::X => [length, 0.0, 0.0],
            GizmoAxis::Y => [0.0, length, 0.0],
            GizmoAxis::Z => [0.0, 0.0, length],
            _ => [0.0, 0.0, 0.0],
        };

        let apex_normal = match axis {
            GizmoAxis::X => [1.0, 0.0, 0.0],
            GizmoAxis::Y => [0.0, 1.0, 0.0],
            GizmoAxis::Z => [0.0, 0.0, 1.0],
            _ => [0.0, 0.0, 0.0],
        };

        vertices.push(Vertex::new(apex, apex_normal).with_color(color));
        let apex_idx = cone_base;

        // Cone base circle vertices
        let cone_circle_start = vertices.len() as u32;
        let slope = head_radius / head_length;
        let normal_len = (1.0 + slope * slope).sqrt();
        let normal_along = 1.0 / normal_len;
        let normal_out = slope / normal_len;

        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let (pos, normal) = match axis {
                GizmoAxis::X => (
                    [shaft_length, head_radius * cos_t, head_radius * sin_t],
                    [normal_along, normal_out * cos_t, normal_out * sin_t],
                ),
                GizmoAxis::Y => (
                    [head_radius * cos_t, shaft_length, head_radius * sin_t],
                    [normal_out * cos_t, normal_along, normal_out * sin_t],
                ),
                GizmoAxis::Z => (
                    [head_radius * cos_t, head_radius * sin_t, shaft_length],
                    [normal_out * cos_t, normal_out * sin_t, normal_along],
                ),
                _ => continue,
            };

            vertices.push(Vertex::new(pos, normal).with_color(color));
        }

        // Cone side indices
        for i in 0..segments {
            indices.push(apex_idx);
            indices.push(cone_circle_start + i + 1);
            indices.push(cone_circle_start + i);
        }

        // Cone base cap (facing back)
        let cap_center_idx = vertices.len() as u32;
        let cap_normal = match axis {
            GizmoAxis::X => [-1.0, 0.0, 0.0],
            GizmoAxis::Y => [0.0, -1.0, 0.0],
            GizmoAxis::Z => [0.0, 0.0, -1.0],
            _ => [0.0, 0.0, 0.0],
        };

        let cap_center = match axis {
            GizmoAxis::X => [shaft_length, 0.0, 0.0],
            GizmoAxis::Y => [0.0, shaft_length, 0.0],
            GizmoAxis::Z => [0.0, 0.0, shaft_length],
            _ => [0.0, 0.0, 0.0],
        };

        vertices.push(Vertex::new(cap_center, cap_normal).with_color(color));

        let cap_start = vertices.len() as u32;
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let pos = match axis {
                GizmoAxis::X => [shaft_length, head_radius * cos_t, head_radius * sin_t],
                GizmoAxis::Y => [head_radius * cos_t, shaft_length, head_radius * sin_t],
                GizmoAxis::Z => [head_radius * cos_t, head_radius * sin_t, shaft_length],
                _ => continue,
            };

            vertices.push(Vertex::new(pos, cap_normal).with_color(color));
        }

        // Cap indices (reversed winding for back-facing)
        for i in 0..segments {
            indices.push(cap_center_idx);
            indices.push(cap_start + i);
            indices.push(cap_start + i + 1);
        }

        MeshData {
            vertices,
            indices: Some(indices),
        }
    }

    /// Upload all gizmo meshes to GPU
    pub fn upload(&self, device: &Device) -> GpuTranslateGizmo {
        GpuTranslateGizmo {
            x_axis: self.x_axis.upload(device),
            y_axis: self.y_axis.upload(device),
            z_axis: self.z_axis.upload(device),
        }
    }
}

/// GPU-uploaded translation gizmo
pub struct GpuTranslateGizmo {
    pub x_axis: GpuMesh,
    pub y_axis: GpuMesh,
    pub z_axis: GpuMesh,
}

/// Scale gizmo mesh data
/// Creates cubes at the end of each axis
pub struct ScaleGizmo {
    pub x_axis: MeshData,
    pub y_axis: MeshData,
    pub z_axis: MeshData,
    pub center: MeshData,
}

impl ScaleGizmo {
    /// Create scale gizmo with given dimensions
    pub fn new(length: f32, cube_size: f32, shaft_radius: f32) -> Self {
        Self {
            x_axis: Self::create_axis_with_cube(length, cube_size, shaft_radius, GizmoAxis::X),
            y_axis: Self::create_axis_with_cube(length, cube_size, shaft_radius, GizmoAxis::Y),
            z_axis: Self::create_axis_with_cube(length, cube_size, shaft_radius, GizmoAxis::Z),
            center: Self::create_center_cube(cube_size * 0.8),
        }
    }

    /// Create default-sized scale gizmo
    pub fn default_size() -> Self {
        Self::new(1.0, 0.1, 0.02)
    }

    fn create_axis_with_cube(length: f32, cube_size: f32, shaft_radius: f32, axis: GizmoAxis) -> MeshData {
        use std::f32::consts::PI;

        let color = axis.color();
        let segments = 8u32;
        let shaft_length = length - cube_size / 2.0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Shaft (same as translate gizmo)
        for i in 0..=segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let (start, end, normal) = match axis {
                GizmoAxis::X => (
                    [0.0, shaft_radius * cos_t, shaft_radius * sin_t],
                    [shaft_length, shaft_radius * cos_t, shaft_radius * sin_t],
                    [0.0, cos_t, sin_t],
                ),
                GizmoAxis::Y => (
                    [shaft_radius * cos_t, 0.0, shaft_radius * sin_t],
                    [shaft_radius * cos_t, shaft_length, shaft_radius * sin_t],
                    [cos_t, 0.0, sin_t],
                ),
                GizmoAxis::Z => (
                    [shaft_radius * cos_t, shaft_radius * sin_t, 0.0],
                    [shaft_radius * cos_t, shaft_radius * sin_t, shaft_length],
                    [cos_t, sin_t, 0.0],
                ),
                _ => continue,
            };

            vertices.push(Vertex::new(start, normal).with_color(color));
            vertices.push(Vertex::new(end, normal).with_color(color));
        }

        for i in 0..segments {
            let base = i * 2;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);
        }

        // Add cube at end
        let cube_offset = match axis {
            GizmoAxis::X => [length, 0.0, 0.0],
            GizmoAxis::Y => [0.0, length, 0.0],
            GizmoAxis::Z => [0.0, 0.0, length],
            _ => [0.0, 0.0, 0.0],
        };

        let s = cube_size / 2.0;
        let cube_base = vertices.len() as u32;

        // Cube vertices (centered at cube_offset)
        let cube_verts = [
            // Front face (+Z local)
            ([-s, -s, s], [0.0, 0.0, 1.0]),
            ([s, -s, s], [0.0, 0.0, 1.0]),
            ([s, s, s], [0.0, 0.0, 1.0]),
            ([-s, s, s], [0.0, 0.0, 1.0]),
            // Back face
            ([s, -s, -s], [0.0, 0.0, -1.0]),
            ([-s, -s, -s], [0.0, 0.0, -1.0]),
            ([-s, s, -s], [0.0, 0.0, -1.0]),
            ([s, s, -s], [0.0, 0.0, -1.0]),
            // Top face
            ([-s, s, s], [0.0, 1.0, 0.0]),
            ([s, s, s], [0.0, 1.0, 0.0]),
            ([s, s, -s], [0.0, 1.0, 0.0]),
            ([-s, s, -s], [0.0, 1.0, 0.0]),
            // Bottom face
            ([-s, -s, -s], [0.0, -1.0, 0.0]),
            ([s, -s, -s], [0.0, -1.0, 0.0]),
            ([s, -s, s], [0.0, -1.0, 0.0]),
            ([-s, -s, s], [0.0, -1.0, 0.0]),
            // Right face
            ([s, -s, s], [1.0, 0.0, 0.0]),
            ([s, -s, -s], [1.0, 0.0, 0.0]),
            ([s, s, -s], [1.0, 0.0, 0.0]),
            ([s, s, s], [1.0, 0.0, 0.0]),
            // Left face
            ([-s, -s, -s], [-1.0, 0.0, 0.0]),
            ([-s, -s, s], [-1.0, 0.0, 0.0]),
            ([-s, s, s], [-1.0, 0.0, 0.0]),
            ([-s, s, -s], [-1.0, 0.0, 0.0]),
        ];

        for (pos, normal) in cube_verts {
            let offset_pos = [
                pos[0] + cube_offset[0],
                pos[1] + cube_offset[1],
                pos[2] + cube_offset[2],
            ];
            vertices.push(Vertex::new(offset_pos, normal).with_color(color));
        }

        let cube_indices: [u32; 36] = [
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,   // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        for idx in cube_indices {
            indices.push(cube_base + idx);
        }

        MeshData {
            vertices,
            indices: Some(indices),
        }
    }

    fn create_center_cube(size: f32) -> MeshData {
        MeshData::cube(size, GizmoAxis::All.color())
    }

    /// Upload all gizmo meshes to GPU
    pub fn upload(&self, device: &Device) -> GpuScaleGizmo {
        GpuScaleGizmo {
            x_axis: self.x_axis.upload(device),
            y_axis: self.y_axis.upload(device),
            z_axis: self.z_axis.upload(device),
            center: self.center.upload(device),
        }
    }
}

/// GPU-uploaded scale gizmo
pub struct GpuScaleGizmo {
    pub x_axis: GpuMesh,
    pub y_axis: GpuMesh,
    pub z_axis: GpuMesh,
    pub center: GpuMesh,
}

/// Combined gizmo that can switch between modes
pub struct Gizmo {
    pub translate: TranslateGizmo,
    pub scale: ScaleGizmo,
}

impl Gizmo {
    pub fn new() -> Self {
        Self {
            translate: TranslateGizmo::default_size(),
            scale: ScaleGizmo::default_size(),
        }
    }

    pub fn upload(&self, device: &Device) -> GpuGizmo {
        GpuGizmo {
            translate: self.translate.upload(device),
            scale: self.scale.upload(device),
        }
    }
}

impl Default for Gizmo {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU-uploaded gizmo with all modes
pub struct GpuGizmo {
    pub translate: GpuTranslateGizmo,
    pub scale: GpuScaleGizmo,
}
