//! Application state management
//!
//! Central state container that holds all application data.
//! Uses RwLock for thread-safe access from Tauri commands.

use std::sync::RwLock;

use cadhy_commands::CommandStack;
use cadhy_viewport::{Camera, OffscreenRenderer, Scene, ViewMode};

/// Central application state
///
/// This is managed by Tauri and accessed via `State<AppState>` in commands.
/// All fields use RwLock for safe concurrent access.
pub struct AppState {
    /// The 3D scene graph
    pub scene: RwLock<Scene>,

    /// Command history for undo/redo
    pub commands: RwLock<CommandStack>,

    /// Whether the scene needs to be re-rendered
    pub dirty: RwLock<bool>,

    /// Current viewport size
    pub viewport_size: RwLock<(u32, u32)>,

    /// Current camera state (serializable for IPC)
    pub camera_state: RwLock<CameraState>,

    /// Offscreen renderer (initialized lazily)
    pub renderer: RwLock<Option<OffscreenRenderer>>,

    /// Current view mode (solid/wireframe)
    pub view_mode: RwLock<ViewMode>,
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
            scene: RwLock::new(Scene::new()),
            commands: RwLock::new(CommandStack::new()),
            dirty: RwLock::new(false),
            viewport_size: RwLock::new((1280, 720)),
            camera_state: RwLock::new(CameraState::default()),
            renderer: RwLock::new(None),
            view_mode: RwLock::new(ViewMode::Solid),
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

    /// Initialize the offscreen renderer
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

    /// Check if renderer is initialized
    pub fn has_renderer(&self) -> bool {
        self.renderer
            .read()
            .map(|r| r.is_some())
            .unwrap_or(false)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
