use super::transform::Transform;
use uuid::Uuid;

use crate::mesh::GpuMesh;

/// Unique identifier for scene objects
pub type ObjectId = Uuid;

/// A renderable object in the scene
pub struct SceneObject {
    pub id: ObjectId,
    pub name: String,
    pub transform: Transform,
    pub mesh: Option<GpuMesh>,
    pub visible: bool,
    pub selected: bool,
    /// Object-specific color for picking
    pub pick_color: [u8; 4],
}

impl SceneObject {
    pub fn new(name: impl Into<String>) -> Self {
        let id = Uuid::new_v4();
        // Generate pick color from UUID (unique per object)
        let bytes = id.as_bytes();
        let pick_color = [bytes[0], bytes[1], bytes[2], 255];

        Self {
            id,
            name: name.into(),
            transform: Transform::default(),
            mesh: None,
            visible: true,
            selected: false,
            pick_color,
        }
    }

    pub fn with_mesh(mut self, mesh: GpuMesh) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}