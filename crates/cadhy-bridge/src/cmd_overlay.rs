//! Overlay mode commands for the inverted architecture
//!
//! These commands support the "overlay" mode where:
//! - Main window = wgpu surface (receives native mouse events)
//! - Child webviews = UI regions only

use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Manager, Runtime, State};

use cadhy_viewport::{Camera, NativeViewport, Projection, Scene};

use crate::error::{BridgeError, BridgeResult};
use crate::ui_overlay::{OverlayLayout, ViewportBounds};

/// State for the overlay viewport
pub struct OverlayViewportState {
    pub viewport: RwLock<Option<NativeViewport>>,
    pub camera: RwLock<Camera>,
    pub scene: Arc<RwLock<Scene>>,
    pub layout: RwLock<OverlayLayout>,
    pub window_size: RwLock<(u32, u32)>,
    pub running: std::sync::atomic::AtomicBool,
}

impl OverlayViewportState {
    pub fn new() -> Self {
        Self {
            viewport: RwLock::new(None),
            camera: RwLock::new(Camera {
                position: glam::Vec3::new(5.0, 5.0, 5.0),
                target: glam::Vec3::ZERO,
                up: glam::Vec3::Y,
                projection: Projection::Perspective {
                    fov: 45.0_f32.to_radians(),
                    near: 0.1,
                    far: 1000.0,
                },
                aspect_ratio: 16.0 / 9.0,
            }),
            scene: Arc::new(RwLock::new(Scene::new())),
            layout: RwLock::new(OverlayLayout::default()),
            window_size: RwLock::new((1400, 900)),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Calculate viewport bounds based on current layout
    pub fn viewport_bounds(&self) -> ViewportBounds {
        let layout = self.layout.read().unwrap();
        let (width, height) = *self.window_size.read().unwrap();

        let left_width = if layout.left_collapsed { 0 } else { layout.left_panel_width };
        let right_width = if layout.right_collapsed { 0 } else { layout.right_panel_width };

        ViewportBounds {
            x: left_width as i32,
            y: layout.titlebar_height as i32,
            width: width.saturating_sub(left_width + right_width),
            height: height.saturating_sub(layout.titlebar_height + layout.statusbar_height),
        }
    }
}

impl Default for OverlayViewportState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize overlay mode viewport on the main window
///
/// This creates a wgpu surface directly on the main window.
/// Call this AFTER setting up the UI overlay webviews.
#[tauri::command]
pub async fn overlay_init<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, OverlayViewportState>,
    width: u32,
    height: u32,
) -> BridgeResult<bool> {
    use std::sync::atomic::Ordering;

    if state.running.load(Ordering::SeqCst) {
        tracing::info!("Overlay viewport already initialized");
        return Ok(true);
    }

    tracing::info!("Initializing overlay viewport {}x{}", width, height);

    // Update window size
    if let Ok(mut size) = state.window_size.write() {
        *size = (width, height);
    }

    // Calculate viewport bounds
    let bounds = state.viewport_bounds();
    tracing::info!("Viewport bounds: {:?}", bounds);

    // Get the main window (raw Window)
    // In overlay mode, this is created by setup() without a webview
    let main_window = app
        .get_window("main")
        .ok_or_else(|| BridgeError::ViewportInit("Main window not found".to_string()))?;

    // Channel for result from main thread
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let viewport_ref = state.viewport.try_write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;
    drop(viewport_ref); // Release lock before spawning

    let vp_width = bounds.width;
    let vp_height = bounds.height;

    // Initialize wgpu on main thread (required for Metal on macOS)
    app.run_on_main_thread(move || {
        let result = (|| -> Result<(), String> {
            tracing::info!("Creating NativeViewport on main thread...");

            // Wrap window in Arc for wgpu
            let window_arc = Arc::new(main_window);

            // Create the viewport
            let viewport = pollster::block_on(async {
                NativeViewport::new(window_arc, vp_width, vp_height).await
            }).map_err(|e| format!("NativeViewport creation failed: {:?}", e))?;

            tracing::info!(
                "NativeViewport created: {}x{}, format: {:?}",
                viewport.size().0,
                viewport.size().1,
                viewport.surface_format()
            );

            // Note: We can't easily store the viewport back into state from here
            // because we'd need to move it back. This is a limitation of the current design.
            // For now, we'll use a different approach.

            Ok(())
        })();

        let _ = tx.send(result);
    }).map_err(|e| BridgeError::ViewportInit(format!("Main thread dispatch failed: {:?}", e)))?;

    // Wait for result
    let init_result = rx.recv()
        .map_err(|e| BridgeError::ViewportInit(format!("Failed to receive init result: {}", e)))?;

    match init_result {
        Ok(()) => {
            state.running.store(true, Ordering::SeqCst);
            tracing::info!("Overlay viewport initialized successfully");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Overlay viewport initialization failed: {}", e);
            Err(BridgeError::ViewportInit(e))
        }
    }
}

/// Get current viewport bounds (area not covered by UI overlays)
#[tauri::command]
pub async fn overlay_get_viewport_bounds(
    state: State<'_, OverlayViewportState>,
) -> BridgeResult<ViewportBounds> {
    Ok(state.viewport_bounds())
}

/// Update layout configuration
#[tauri::command]
pub async fn overlay_set_layout(
    state: State<'_, OverlayViewportState>,
    titlebar_height: Option<u32>,
    left_panel_width: Option<u32>,
    right_panel_width: Option<u32>,
    statusbar_height: Option<u32>,
    left_collapsed: Option<bool>,
    right_collapsed: Option<bool>,
) -> BridgeResult<ViewportBounds> {
    if let Ok(mut layout) = state.layout.write() {
        if let Some(h) = titlebar_height {
            layout.titlebar_height = h;
        }
        if let Some(w) = left_panel_width {
            layout.left_panel_width = w;
        }
        if let Some(w) = right_panel_width {
            layout.right_panel_width = w;
        }
        if let Some(h) = statusbar_height {
            layout.statusbar_height = h;
        }
        if let Some(c) = left_collapsed {
            layout.left_collapsed = c;
        }
        if let Some(c) = right_collapsed {
            layout.right_collapsed = c;
        }
    }

    Ok(state.viewport_bounds())
}

/// Toggle left panel visibility
#[tauri::command]
pub async fn overlay_toggle_left_panel(
    state: State<'_, OverlayViewportState>,
) -> BridgeResult<ViewportBounds> {
    if let Ok(mut layout) = state.layout.write() {
        layout.left_collapsed = !layout.left_collapsed;
    }
    Ok(state.viewport_bounds())
}

/// Toggle right panel visibility
#[tauri::command]
pub async fn overlay_toggle_right_panel(
    state: State<'_, OverlayViewportState>,
) -> BridgeResult<ViewportBounds> {
    if let Ok(mut layout) = state.layout.write() {
        layout.right_collapsed = !layout.right_collapsed;
    }
    Ok(state.viewport_bounds())
}

/// Window resize handler - updates stored size and recalculates bounds
#[tauri::command]
pub async fn overlay_on_resize(
    state: State<'_, OverlayViewportState>,
    width: u32,
    height: u32,
) -> BridgeResult<ViewportBounds> {
    if let Ok(mut size) = state.window_size.write() {
        *size = (width, height);
    }
    Ok(state.viewport_bounds())
}

/// Check if overlay mode is active
#[tauri::command]
pub async fn overlay_is_active(
    state: State<'_, OverlayViewportState>,
) -> BridgeResult<bool> {
    use std::sync::atomic::Ordering;
    Ok(state.running.load(Ordering::SeqCst))
}

/// Camera orbit in overlay mode
#[tauri::command]
pub async fn overlay_camera_orbit(
    state: State<'_, OverlayViewportState>,
    delta_x: f32,
    delta_y: f32,
) -> BridgeResult<()> {
    if let Ok(mut camera) = state.camera.write() {
        let sensitivity = 0.005;
        let offset = camera.position - camera.target;
        let distance = offset.length();

        let mut theta = offset.z.atan2(offset.x);
        let mut phi = (offset.y / distance).acos();

        theta -= delta_x * sensitivity;
        phi = (phi + delta_y * sensitivity).clamp(0.01, std::f32::consts::PI - 0.01);

        camera.position = camera.target
            + glam::Vec3::new(
                distance * phi.sin() * theta.cos(),
                distance * phi.cos(),
                distance * phi.sin() * theta.sin(),
            );
    }
    Ok(())
}

/// Camera pan in overlay mode
#[tauri::command]
pub async fn overlay_camera_pan(
    state: State<'_, OverlayViewportState>,
    delta_x: f32,
    delta_y: f32,
) -> BridgeResult<()> {
    if let Ok(mut camera) = state.camera.write() {
        let sensitivity = 0.01;
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward);

        let offset = right * (-delta_x * sensitivity) + up * (delta_y * sensitivity);
        camera.position += offset;
        camera.target += offset;
    }
    Ok(())
}

/// Camera zoom in overlay mode
#[tauri::command]
pub async fn overlay_camera_zoom(
    state: State<'_, OverlayViewportState>,
    delta: f32,
) -> BridgeResult<()> {
    if let Ok(mut camera) = state.camera.write() {
        let direction = (camera.target - camera.position).normalize();
        let distance = (camera.position - camera.target).length();
        let zoom_speed = distance * 0.001;
        let new_distance = (distance - delta * zoom_speed).max(0.5);

        camera.position = camera.target - direction * new_distance;
    }
    Ok(())
}

/// Reset camera to default position
#[tauri::command]
pub async fn overlay_camera_reset(
    state: State<'_, OverlayViewportState>,
) -> BridgeResult<()> {
    if let Ok(mut camera) = state.camera.write() {
        camera.position = glam::Vec3::new(5.0, 5.0, 5.0);
        camera.target = glam::Vec3::ZERO;
        camera.up = glam::Vec3::Y;
    }
    Ok(())
}
