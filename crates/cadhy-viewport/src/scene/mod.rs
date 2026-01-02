//! Scene graph for the viewport
//!
//! Contains all renderable objects, materials, and scene state.

mod object;
mod scene;
mod transform;

pub use object::{ObjectId, SceneObject};
pub use scene::Scene;
pub use transform::Transform;