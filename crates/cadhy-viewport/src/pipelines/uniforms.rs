//! GPU Uniforms for camera and model transforms
//!
//! Provides uniform buffers and bind groups for passing transform data to shaders.

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat4, Vec3};
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};

// ============================================================================
// CAMERA UNIFORM
// ============================================================================

/// Camera uniform data for GPU
/// Layout matches the WGSL struct CameraUniform
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    /// View-projection matrix (combined view and projection)
    pub view_proj: [[f32; 4]; 4],
    /// Camera position in world space (for lighting calculations)
    pub view_pos: [f32; 3],
    /// Padding to align to 16 bytes
    pub _padding: f32,
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: [0.0, 0.0, 0.0],
            _padding: 0.0,
        }
    }
}

impl CameraUniform {
    /// Update from camera matrices
    pub fn from_matrices(view_proj: Mat4, camera_pos: Vec3) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            view_pos: camera_pos.to_array(),
            _padding: 0.0,
        }
    }
}

/// Camera uniform buffer with bind group
pub struct CameraBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl CameraBuffer {
    /// Create a new camera buffer
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    /// Update the buffer with new camera data
    pub fn update(&self, queue: &Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
    }

    /// Create the bind group layout for camera uniform
    pub fn create_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

// ============================================================================
// MODEL UNIFORM
// ============================================================================

/// Model uniform data for GPU
/// Layout matches the WGSL struct ModelUniform
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModelUniform {
    /// Model matrix (local -> world)
    pub model: [[f32; 4]; 4],
    /// Normal matrix for transforming normals (inverse transpose of model 3x3)
    /// Stored as 3 vec4s for proper alignment (WGSL mat3x3 requires vec4 padding)
    pub normal_matrix: [[f32; 4]; 3],
}

impl Default for ModelUniform {
    fn default() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }
}

impl ModelUniform {
    /// Create from a model matrix
    pub fn from_matrix(model: Mat4) -> Self {
        // Extract 3x3 rotation/scale matrix and compute inverse transpose for normals
        let normal_matrix = Mat3::from_mat4(model).inverse().transpose();
        let cols = normal_matrix.to_cols_array();
        
        Self {
            model: model.to_cols_array_2d(),
            normal_matrix: [
                [cols[0], cols[1], cols[2], 0.0],
                [cols[3], cols[4], cols[5], 0.0],
                [cols[6], cols[7], cols[8], 0.0],
            ],
        }
    }
}

/// Model uniform buffer with bind group
pub struct ModelBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl ModelBuffer {
    /// Create a new model buffer
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Uniform Buffer"),
            size: std::mem::size_of::<ModelUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    /// Update the buffer with new model data
    pub fn update(&self, queue: &Queue, uniform: &ModelUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
    }

    /// Create the bind group layout for model uniform
    pub fn create_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_uniform_size() {
        // Camera uniform should be 80 bytes (64 for mat4 + 12 for vec3 + 4 padding)
        assert_eq!(std::mem::size_of::<CameraUniform>(), 80);
    }

    #[test]
    fn test_model_uniform_size() {
        // Model uniform should be 112 bytes (64 for mat4 + 48 for 3 vec4s)
        assert_eq!(std::mem::size_of::<ModelUniform>(), 112);
    }

    #[test]
    fn test_camera_uniform_from_matrices() {
        let view_proj = Mat4::perspective_rh(45.0_f32.to_radians(), 16.0/9.0, 0.1, 100.0);
        let pos = Vec3::new(5.0, 5.0, 5.0);
        let uniform = CameraUniform::from_matrices(view_proj, pos);
        
        assert_eq!(uniform.view_pos, [5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_model_uniform_from_matrix() {
        let model = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let uniform = ModelUniform::from_matrix(model);
        
        // Translation should be in the 4th column
        assert_eq!(uniform.model[3], [1.0, 2.0, 3.0, 1.0]);
    }
}
