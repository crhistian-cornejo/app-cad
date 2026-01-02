//! Application state management
//!
//! Central state container that holds all application data.
//! Uses RwLock for thread-safe access from Tauri commands.
//!
//! # Rendering Modes
//!
//! The application supports two rendering modes:
//!
//! 1. **Embedded Mode** (preferred): Uses an embedded child window with wgpu
//!    direct rendering for 60+ FPS. The child window moves with the parent.
//!
//! 2. **Fallback Mode**: Uses OffscreenRenderer with base64 frame transfer.
//!    Slower but works when native mode isn't available.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cadhy_cad::Shape;
use cadhy_commands::CommandStack;
use cadhy_viewport::{Camera, OffscreenRenderer, Scene, ViewMode};
use uuid::Uuid;

/// Central application state
///
/// This is managed by Tauri and accessed via `State<AppState>` in commands.
/// All fields use RwLock for safe concurrent access.
///
/// Note: EmbeddedViewport is stored separately in the app via manage()
/// because it needs the Runtime type parameter.
pub struct AppState {
    /// The 3D scene graph
    pub scene: Arc<RwLock<Scene>>,

    /// B-Rep shape storage for CAD operations
    /// Maps scene object UUIDs to their OpenCASCADE shapes
    pub shape_store: RwLock<HashMap<Uuid, Shape>>,

    /// Command history for undo/redo
    pub commands: RwLock<CommandStack>,

    /// Whether the scene needs to be re-rendered
    pub dirty: RwLock<bool>,

    /// Current viewport size
    pub viewport_size: RwLock<(u32, u32)>,

    /// Current camera state (serializable for IPC)
    pub camera_state: RwLock<CameraState>,

    /// Offscreen renderer (fallback mode)
    pub renderer: RwLock<Option<OffscreenRenderer>>,

    /// Current view mode (solid/wireframe)
    pub view_mode: RwLock<ViewMode>,

    /// Whether embedded viewport rendering is active
    pub embedded_mode: Arc<RwLock<bool>>,
}

/// Serializable camera state for IPC
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraState {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraState {
    /// Convert to viewport Camera
    pub fn to_camera(&self, aspect_ratio: f32) -> Camera {
        use cadhy_viewport::Projection;
        use glam::Vec3;

        Camera {
            position: Vec3::from_array(self.position),
            target: Vec3::from_array(self.target),
            up: Vec3::from_array(self.up),
            projection: Projection::Perspective {
                fov: self.fov.to_radians(),
                near: self.near,
                far: self.far,
            },
            aspect_ratio,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scene: Arc::new(RwLock::new(Scene::new())),
            shape_store: RwLock::new(HashMap::new()),
            commands: RwLock::new(CommandStack::new()),
            dirty: RwLock::new(false),
            viewport_size: RwLock::new((1280, 720)),
            camera_state: RwLock::new(CameraState::default()),
            renderer: RwLock::new(None),
            view_mode: RwLock::new(ViewMode::Solid),
            embedded_mode: Arc::new(RwLock::new(false)),
        }
    }

    /// Store a B-Rep shape for a scene object
    pub fn store_shape(&self, id: Uuid, shape: Shape) {
        if let Ok(mut store) = self.shape_store.write() {
            store.insert(id, shape);
        }
    }

    /// Get a cloned shape by object ID
    pub fn get_shape(&self, id: Uuid) -> Option<Shape> {
        self.shape_store
            .read()
            .ok()
            .and_then(|store| store.get(&id).cloned())
    }

    /// Remove a shape from storage
    pub fn remove_shape(&self, id: Uuid) {
        if let Ok(mut store) = self.shape_store.write() {
            store.remove(&id);
        }
    }

    /// Mark the scene as needing a re-render
    pub fn mark_dirty(&self) {
        if let Ok(mut dirty) = self.dirty.write() {
            *dirty = true;
        }
    }

    /// Check if scene needs re-render and clear the flag
    pub fn take_dirty(&self) -> bool {
        if let Ok(mut dirty) = self.dirty.write() {
            let was_dirty = *dirty;
            *dirty = false;
            was_dirty
        } else {
            false
        }
    }

    /// Check if scene needs re-render (without clearing)
    pub fn is_dirty(&self) -> bool {
        self.dirty.read().map(|d| *d).unwrap_or(false)
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&self) {
        if let Ok(mut dirty) = self.dirty.write() {
            *dirty = false;
        }
    }

    /// Check if embedded mode is active
    pub fn is_embedded_mode(&self) -> bool {
        self.embedded_mode.read().map(|n| *n).unwrap_or(false)
    }

    /// Set embedded mode active
    pub fn set_embedded_mode(&self, active: bool) {
        if let Ok(mut mode) = self.embedded_mode.write() {
            *mode = active;
        }
    }

    /// Initialize the offscreen renderer (fallback mode)
    pub async fn init_renderer(&self, width: u32, height: u32) -> Result<(), String> {
        let renderer = OffscreenRenderer::new(width, height)
            .await
            .map_err(|e| e.to_string())?;

        let mut renderer_lock = self
            .renderer
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        *renderer_lock = Some(renderer);
        Ok(())
    }

    /// Check if any renderer is initialized
    pub fn has_renderer(&self) -> bool {
        // Check embedded mode first
        if self.is_embedded_mode() {
            return true;
        }
        // Fallback to offscreen
        self.renderer
            .read()
            .map(|r| r.is_some())
            .unwrap_or(false)
    }

    /// Get the shared scene
    pub fn shared_scene(&self) -> Arc<RwLock<Scene>> {
        self.scene.clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
