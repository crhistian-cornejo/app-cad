//! Gizmo render pipeline - renders transform handles on top of scene

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline, TextureFormat};

use super::uniforms::CameraBuffer;

/// Uniform for gizmo position and scale
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GizmoUniform {
    /// World position of gizmo center
    pub position: [f32; 3],
    /// Scale factor (based on camera distance for constant screen size)
    pub scale: f32,
    /// Highlight multiplier for axis (R=X, G=Y, B=Z, A=strength)
    pub highlight_axis: [f32; 4],
}

impl Default for GizmoUniform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            scale: 1.0,
            highlight_axis: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl GizmoUniform {
    pub fn new(position: [f32; 3], scale: f32) -> Self {
        Self {
            position,
            scale,
            highlight_axis: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn with_highlight(mut self, axis: crate::mesh::GizmoAxis) -> Self {
        use crate::mesh::GizmoAxis;
        self.highlight_axis = match axis {
            GizmoAxis::X => [1.0, 0.0, 0.0, 1.0],
            GizmoAxis::Y => [0.0, 1.0, 0.0, 1.0],
            GizmoAxis::Z => [0.0, 0.0, 1.0, 1.0],
            GizmoAxis::XY => [1.0, 1.0, 0.0, 1.0],
            GizmoAxis::XZ => [1.0, 0.0, 1.0, 1.0],
            GizmoAxis::YZ => [0.0, 1.0, 1.0, 1.0],
            GizmoAxis::All => [1.0, 1.0, 1.0, 1.0],
            GizmoAxis::None => [0.0, 0.0, 0.0, 0.0],
        };
        self
    }
}

/// GPU buffer for gizmo uniform
pub struct GizmoBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl GizmoBuffer {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GizmoUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    pub fn update(&self, queue: &Queue, uniform: &GizmoUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*uniform]));
    }

    pub fn create_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gizmo Uniform Layout"),
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

/// Gizmo rendering pipeline
/// Renders on top of scene with depth testing disabled
pub struct GizmoPipeline {
    pub pipeline: RenderPipeline,
    pub camera_layout: BindGroupLayout,
    pub gizmo_layout: BindGroupLayout,
}

impl GizmoPipeline {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gizmo.wgsl").into()),
        });

        // Create bind group layouts
        let camera_layout = CameraBuffer::create_layout(device);
        let gizmo_layout = GizmoBuffer::create_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[&camera_layout, &gizmo_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gizmo Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[crate::mesh::Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Enable alpha blending for semi-transparent gizmo parts
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // IMPORTANT: Depth compare Always means gizmo renders on top
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Don't write to depth buffer
                depth_compare: wgpu::CompareFunction::Always, // Always pass depth test
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            camera_layout,
            gizmo_layout,
        }
    }
}
