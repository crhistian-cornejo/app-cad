//! Render pipelines for the viewport
//!
//! Contains shaders and pipeline configurations for different render modes.

pub mod shaded;
pub mod wireframe;
pub mod picking;
pub mod grid;
pub mod uniforms;

// Re-exports
pub use shaded::ShadedPipeline;
pub use wireframe::WireframePipeline;
pub use picking::PickingPipeline;
pub use grid::GridPipeline;
pub use uniforms::{CameraBuffer, CameraUniform, ModelBuffer, ModelUniform};
