//! Render Context - holds all GPU resources for rendering
//!
//! This is the main rendering context that owns pipelines, uniform buffers,
//! and manages the render loop.

use wgpu::{Device, Queue, TextureFormat};

use crate::camera::Camera;
use crate::mesh::GpuMesh;
use crate::pipelines::{
    CameraBuffer, CameraUniform, GridPipeline, ModelBuffer, ModelUniform, ShadedPipeline,
    WireframePipeline,
};
use crate::scene::Scene;

/// View mode for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Solid,
    Wireframe,
}

/// Render context containing all GPU resources
pub struct RenderContext {
    // Pipelines
    pub shaded_pipeline: ShadedPipeline,
    pub wireframe_pipeline: WireframePipeline,
    pub grid_pipeline: GridPipeline,

    // Uniform buffers (shared between shaded and wireframe)
    pub camera_buffer: CameraBuffer,
    pub model_buffer: ModelBuffer,

    // Wireframe uses same layouts as shaded, so we can reuse buffers
    // Grid camera buffer (uses grid pipeline's layout)
    pub grid_camera_buffer: CameraBuffer,

    // Grid mesh
    pub grid_mesh: GpuMesh,

    // View settings
    pub view_mode: ViewMode,
    pub show_grid: bool,
    pub background_color: wgpu::Color,
}

impl RenderContext {
    /// Create a new render context
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        // Create pipelines
        let shaded_pipeline = ShadedPipeline::new(device, format);
        let wireframe_pipeline = WireframePipeline::new(device, format);
        let grid_pipeline = GridPipeline::new(device, format);

        // Create uniform buffers using shaded pipeline's layouts
        // (wireframe uses compatible layouts)
        let camera_buffer = CameraBuffer::new(device, &shaded_pipeline.camera_layout);
        let model_buffer = ModelBuffer::new(device, &shaded_pipeline.model_layout);

        // Create separate camera buffer for grid (uses its own layout)
        let grid_camera_buffer = CameraBuffer::new(device, &grid_pipeline.camera_layout);

        // Create grid mesh
        let grid_mesh = GpuMesh::grid(device, 100.0, 50); // 100 units, 50 divisions

        Self {
            shaded_pipeline,
            wireframe_pipeline,
            grid_pipeline,
            camera_buffer,
            model_buffer,
            grid_camera_buffer,
            grid_mesh,
            view_mode: ViewMode::Solid,
            show_grid: true,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.12,
                a: 1.0,
            },
        }
    }

    /// Update camera uniforms
    pub fn update_camera(&self, queue: &Queue, camera: &Camera) {
        let uniform =
            CameraUniform::from_matrices(camera.view_projection_matrix(), camera.position);
        self.camera_buffer.update(queue, &uniform);
        self.grid_camera_buffer.update(queue, &uniform);
    }

    /// Update model uniforms for an object
    #[allow(dead_code)]
    pub fn update_model(&self, queue: &Queue, model_matrix: glam::Mat4) {
        let uniform = ModelUniform::from_matrix(model_matrix);
        self.model_buffer.update(queue, &uniform);
    }

    /// Set the view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Toggle grid visibility
    #[allow(dead_code)]
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Render the scene
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        scene: &'a Scene,
        queue: &Queue,
    ) {
        // Draw grid first (background)
        if self.show_grid {
            render_pass.set_pipeline(&self.grid_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.grid_camera_buffer.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.grid_mesh.vertex_buffer.slice(..));
            render_pass.draw(0..self.grid_mesh.vertex_count, 0..1);
        }

        // Select pipeline based on view mode
        let pipeline = match self.view_mode {
            ViewMode::Solid => &self.shaded_pipeline.pipeline,
            ViewMode::Wireframe => &self.wireframe_pipeline.pipeline,
        };

        render_pass.set_pipeline(pipeline);

        // Bind camera
        render_pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);

        // Draw each visible object
        for object in scene.visible_objects() {
            if let Some(mesh) = &object.mesh {
                // Update model matrix for this object
                let model_uniform = ModelUniform::from_matrix(object.transform.matrix());
                queue.write_buffer(
                    &self.model_buffer.buffer,
                    0,
                    bytemuck::bytes_of(&model_uniform),
                );

                // Bind model uniform
                render_pass.set_bind_group(1, &self.model_buffer.bind_group, &[]);

                // Set vertex buffer
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                // Draw
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
    }
}
