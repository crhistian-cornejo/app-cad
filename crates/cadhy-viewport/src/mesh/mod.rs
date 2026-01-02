//! GPU Mesh representation
//!
//! Handles vertex/index buffers for rendering.

mod mesh;
mod vertex;

pub use mesh::{GpuMesh, MeshData};
pub use vertex::Vertex;