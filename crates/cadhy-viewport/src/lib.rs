//! CADHY Viewport - wgpu-based 3D rendering engine
//!
//! This is the core rendering module, equivalent to Blender's draw/ module.
//! It handles all GPU operations: mesh rendering, picking, gizmos, and overlays.

pub mod camera;
pub mod error;
pub mod mesh;
pub mod offscreen;
pub mod picking;
pub mod pipelines;
pub mod render_context;
pub mod renderer;
pub mod scene;

pub use camera::{Camera, CameraController, Projection};
pub use error::{ViewportError, ViewportResult};
pub use mesh::{GpuMesh, MeshData, Vertex};
pub use offscreen::OffscreenRenderer;
pub use picking::{color_to_id, id_to_color, PickingBuffer};
pub use render_context::{RenderContext, ViewMode};
pub use renderer::Renderer;
pub use scene::{ObjectId, Scene, SceneObject, Transform};

/// Re-export wgpu types for external use
pub use wgpu;
