//! Data Transfer Objects for Tauri IPC
//!
//! These structs are used to communicate between React and Rust.
//! They are designed to be serializable and lightweight.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Scene DTOs
// ============================================================================

/// Minimal object info for UI (outliner, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObjectDto {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub selected: bool,
    pub object_type: ObjectType,
}

/// Type of scene object
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectType {
    Mesh,
    Light,
    Camera,
    Empty,
    Curve,
    HydraulicChannel,
    HydraulicStructure,
}

/// Transform data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformDto {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl Default for TransformDto {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
        }
    }
}

// ============================================================================
// Viewport DTOs
// ============================================================================

/// Viewport size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSizeDto {
    pub width: u32,
    pub height: u32,
}

/// Camera orbit input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitInputDto {
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Camera pan input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanInputDto {
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Camera zoom input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomInputDto {
    pub delta: f32,
}

/// Camera state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraDto {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

/// View mode for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ViewModeDto {
    #[default]
    Solid,
    Wireframe,
}

// ============================================================================
// CAD DTOs
// ============================================================================

/// Primitive creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrimitiveParams {
    Box {
        width: f64,
        height: f64,
        depth: f64,
    },
    Sphere {
        radius: f64,
    },
    Cylinder {
        radius: f64,
        height: f64,
    },
    Cone {
        radius: f64,
        height: f64,
    },
    Torus {
        major_radius: f64,
        minor_radius: f64,
    },
    Plane {
        width: f64,
        height: f64,
    },
}

// ============================================================================
// Project DTOs
// ============================================================================

/// Undo/redo state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoStateDto {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_name: Option<String>,
    pub redo_name: Option<String>,
    pub is_modified: bool,
}

// ============================================================================
// Selection DTOs
// ============================================================================

/// Selection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionDto {
    pub ids: Vec<Uuid>,
    pub count: usize,
}
