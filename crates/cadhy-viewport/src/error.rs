//! Viewport error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ViewportError {
    #[error("Failed to create GPU adapter")]
    AdapterCreation,

    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),

    #[error("Surface configuration failed: {0}")]
    SurfaceConfig(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),

    #[error("Buffer creation failed: {0}")]
    BufferCreation(String),

    #[error("Texture creation failed: {0}")]
    TextureCreation(String),

    #[error("Mesh has no vertices")]
    EmptyMesh,

    #[error("Invalid mesh data: {0}")]
    InvalidMesh(String),
}

pub type ViewportResult<T> = Result<T, ViewportError>;
