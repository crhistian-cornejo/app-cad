//! Viewport commands: camera control, resize, render mode
//!
//! Supports two rendering modes:
//! - **Embedded mode**: High-performance wgpu rendering to a child window (60+ FPS)
//! - **Fallback mode**: Offscreen rendering with base64 transfer (for compatibility)

use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Manager, Runtime, State};
use tracing::{error, info};

use cadhy_viewport::ViewMode;

use crate::dto::{CameraDto, OrbitInputDto, PanInputDto, ViewModeDto, ViewportSizeDto, ZoomInputDto};
use crate::embedded_viewport::{EmbeddedViewport, RenderMessage};
use crate::error::BridgeResult;
use crate::state::{AppState, CameraState};

/// Viewport settings DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewportSettingsDto {
    pub show_grid: bool,
    pub show_axes: bool,
    pub anti_aliasing: bool,
}

/// Initialize the offscreen renderer
#[tauri::command]
pub async fn viewport_init(state: State<'_, AppState>, width: u32, height: u32) -> BridgeResult<bool> {
    info!("Initializing viewport renderer {}x{}", width, height);
    
    match state.init_renderer(width, height).await {
        Ok(_) => {
            info!("Viewport renderer initialized successfully");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to initialize viewport renderer: {}", e);
            Err(crate::error::BridgeError::ViewportInit(e))
        }
    }
}

/// Check if renderer is initialized
#[tauri::command]
pub async fn viewport_is_initialized(state: State<'_, AppState>) -> BridgeResult<bool> {
    Ok(state.has_renderer())
}

/// Render a frame and return as base64 encoded PNG
#[tauri::command]
pub async fn viewport_render_frame(state: State<'_, AppState>) -> BridgeResult<String> {
    use base64::Engine;

    // Get renderer and render using the synchronous method
    // This avoids conflicts between pollster and tokio runtimes
    let (rgba_data, actual_size) = {
        let renderer_opt = state
            .renderer
            .read()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        let renderer = renderer_opt
            .as_ref()
            .ok_or_else(|| crate::error::BridgeError::ViewportInit("Renderer not initialized".to_string()))?;

        // Get the actual size from the renderer to ensure consistency
        let size = renderer.size();

        // Create camera with correct aspect ratio from actual renderer size
        let camera = {
            let cam_state = state
                .camera_state
                .read()
                .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
            let aspect = size.0 as f32 / size.1.max(1) as f32;
            cam_state.to_camera(aspect)
        };

        let scene = state
            .scene
            .read()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        // Use synchronous version to avoid runtime conflicts
        let data = renderer.render_to_bytes_sync(&scene, &camera)
            .map_err(|e| crate::error::BridgeError::Render(e.to_string()))?;

        (data, size)
    };

    // Encode as base64 (raw RGBA for now, frontend will display it)
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&rgba_data);

    // Use actual renderer size to ensure width*height*4 == rgba_data.len()
    Ok(format!("data:image/rgba;width={};height={};base64,{}", actual_size.0, actual_size.1, base64_data))
}

/// Resize the viewport
#[tauri::command]
pub async fn viewport_resize(
    state: State<'_, AppState>,
    width: u32,
    height: u32,
) -> BridgeResult<()> {
    // Skip invalid sizes
    if width == 0 || height == 0 {
        return Ok(());
    }

    // Update stored viewport size
    {
        let mut size = state
            .viewport_size
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
        *size = (width, height);
    }

    // Also resize the offscreen renderer if initialized
    {
        let mut renderer_opt = state
            .renderer
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        if let Some(renderer) = renderer_opt.as_mut() {
            renderer.resize(width, height);
        }
    }

    state.mark_dirty();
    Ok(())
}

/// Get current viewport size
#[tauri::command]
pub async fn viewport_get_size(state: State<'_, AppState>) -> BridgeResult<ViewportSizeDto> {
    let size = state
        .viewport_size
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
    Ok(ViewportSizeDto {
        width: size.0,
        height: size.1,
    })
}

/// Set camera state
#[tauri::command]
pub async fn viewport_set_camera(
    state: State<'_, AppState>,
    camera: CameraDto,
) -> BridgeResult<()> {
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    *cam = CameraState {
        position: camera.position,
        target: camera.target,
        up: camera.up,
        fov: camera.fov,
        near: camera.near,
        far: camera.far,
    };

    state.mark_dirty();
    Ok(())
}

/// Get current camera state
#[tauri::command]
pub async fn viewport_get_camera(state: State<'_, AppState>) -> BridgeResult<CameraDto> {
    let cam = state
        .camera_state
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Apply orbit rotation to camera
#[tauri::command]
pub async fn viewport_orbit(
    state: State<'_, AppState>,
    input: OrbitInputDto,
) -> BridgeResult<CameraDto> {
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    // Simple orbit implementation using glam
    use glam::{Mat4, Vec3};

    let position = Vec3::from_array(cam.position);
    let target = Vec3::from_array(cam.target);

    // Calculate orbit
    let to_camera = position - target;
    let distance = to_camera.length();

    // Horizontal rotation (around Y axis)
    let rotation_y = Mat4::from_rotation_y(-input.delta_x * 0.01);

    // Get right vector for vertical rotation
    let up = Vec3::from_array(cam.up);
    let forward = to_camera.normalize();
    let right = up.cross(forward).normalize();

    // Vertical rotation (around right axis)
    let rotation_x = Mat4::from_axis_angle(right, -input.delta_y * 0.01);

    // Apply rotations
    let new_direction = rotation_x.transform_vector3(rotation_y.transform_vector3(to_camera));
    let new_position = target + new_direction.normalize() * distance;

    cam.position = new_position.to_array();

    state.mark_dirty();

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Apply pan movement to camera
#[tauri::command]
pub async fn viewport_pan(state: State<'_, AppState>, input: PanInputDto) -> BridgeResult<CameraDto> {
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    use glam::Vec3;

    let position = Vec3::from_array(cam.position);
    let target = Vec3::from_array(cam.target);
    let up = Vec3::from_array(cam.up);

    // Calculate right and up vectors
    let forward = (target - position).normalize();
    let right = forward.cross(up).normalize();
    let camera_up = right.cross(forward).normalize();

    // Pan speed based on distance to target
    let distance = (target - position).length();
    let pan_speed = distance * 0.002;

    // Calculate pan offset
    let offset = right * (-input.delta_x * pan_speed) + camera_up * (input.delta_y * pan_speed);

    cam.position = (position + offset).to_array();
    cam.target = (target + offset).to_array();

    state.mark_dirty();

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Apply zoom to camera
#[tauri::command]
pub async fn viewport_zoom(
    state: State<'_, AppState>,
    input: ZoomInputDto,
) -> BridgeResult<CameraDto> {
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    use glam::Vec3;

    let position = Vec3::from_array(cam.position);
    let target = Vec3::from_array(cam.target);

    let to_target = target - position;
    let distance = to_target.length();

    // Zoom speed - exponential for smooth zooming
    let zoom_factor = 1.0 - input.delta * 0.001;
    let new_distance = (distance * zoom_factor).max(0.1).min(10000.0);

    let direction = to_target.normalize();
    let new_position = target - direction * new_distance;

    cam.position = new_position.to_array();

    state.mark_dirty();

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Frame camera to fit selected objects
#[tauri::command]
pub async fn viewport_frame_selection(state: State<'_, AppState>) -> BridgeResult<CameraDto> {
    // Get selected objects and calculate bounding box
    let is_selection_empty = {
        let scene = state
            .scene
            .read()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
        scene.selected().is_empty()
    };

    if is_selection_empty {
        // No selection, frame all
        return viewport_frame_all(state).await;
    }

    // TODO: Calculate bounding box of selected objects
    // For now, just return current camera
    let cam = state
        .camera_state
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Frame camera to fit all objects
#[tauri::command]
pub async fn viewport_frame_all(state: State<'_, AppState>) -> BridgeResult<CameraDto> {
    // TODO: Calculate bounding box of all objects
    // For now, reset to default view
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    *cam = CameraState::default();

    state.mark_dirty();

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Set the view mode (solid/wireframe)
#[tauri::command]
pub async fn viewport_set_view_mode(
    state: State<'_, AppState>,
    mode: ViewModeDto,
) -> BridgeResult<()> {
    // Update state
    {
        let mut view_mode = state
            .view_mode
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        *view_mode = match mode {
            ViewModeDto::Solid => ViewMode::Solid,
            ViewModeDto::Wireframe => ViewMode::Wireframe,
        };
    }

    // Update renderer's render context
    {
        let mut renderer_opt = state
            .renderer
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        if let Some(renderer) = renderer_opt.as_mut() {
            let new_mode = match mode {
                ViewModeDto::Solid => ViewMode::Solid,
                ViewModeDto::Wireframe => ViewMode::Wireframe,
            };
            renderer.render_context.set_view_mode(new_mode);
        }
    }

    state.mark_dirty();
    info!("View mode set to {:?}", mode);
    Ok(())
}

/// Get the current view mode
#[tauri::command]
pub async fn viewport_get_view_mode(state: State<'_, AppState>) -> BridgeResult<ViewModeDto> {
    let view_mode = state
        .view_mode
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    Ok(match *view_mode {
        ViewMode::Solid => ViewModeDto::Solid,
        ViewMode::Wireframe => ViewModeDto::Wireframe,
    })
}

/// Reset the camera to default position
#[tauri::command]
pub async fn viewport_reset_camera(state: State<'_, AppState>) -> BridgeResult<CameraDto> {
    let mut cam = state
        .camera_state
        .write()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    *cam = CameraState::default();
    state.mark_dirty();

    Ok(CameraDto {
        position: cam.position,
        target: cam.target,
        up: cam.up,
        fov: cam.fov,
        near: cam.near,
        far: cam.far,
    })
}

/// Set viewport settings (grid, axes, anti-aliasing)
#[tauri::command]
pub async fn viewport_set_settings(
    state: State<'_, AppState>,
    settings: ViewportSettingsDto,
) -> BridgeResult<()> {
    // Update renderer settings
    {
        let mut renderer_opt = state
            .renderer
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

        if let Some(renderer) = renderer_opt.as_mut() {
            renderer.render_context.show_grid = settings.show_grid;
            // TODO: Implement show_axes when axes pipeline is added
            // TODO: Implement anti_aliasing when MSAA is supported
        }
    }

    state.mark_dirty();
    info!("Viewport settings updated: grid={}, axes={}, aa={}",
          settings.show_grid, settings.show_axes, settings.anti_aliasing);
    Ok(())
}

/// Get current viewport settings
#[tauri::command]
pub async fn viewport_get_settings(state: State<'_, AppState>) -> BridgeResult<ViewportSettingsDto> {
    let renderer_opt = state
        .renderer
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    if let Some(renderer) = renderer_opt.as_ref() {
        Ok(ViewportSettingsDto {
            show_grid: renderer.render_context.show_grid,
            show_axes: false, // TODO: Add when implemented
            anti_aliasing: false, // TODO: Add when implemented
        })
    } else {
        Ok(ViewportSettingsDto {
            show_grid: true,
            show_axes: true,
            anti_aliasing: true,
        })
    }
}

/// Check if viewport needs re-render (dirty flag optimization)
#[tauri::command]
pub async fn viewport_is_dirty(state: State<'_, AppState>) -> BridgeResult<bool> {
    Ok(state.is_dirty())
}

/// Clear the dirty flag after rendering
#[tauri::command]
pub async fn viewport_clear_dirty(state: State<'_, AppState>) -> BridgeResult<()> {
    state.clear_dirty();
    Ok(())
}

// ============================================================================
// EMBEDDED VIEWPORT COMMANDS (High-performance mode)
// ============================================================================

/// Type alias for the managed embedded viewport state
type EmbeddedViewportState<R> = Arc<RwLock<Option<EmbeddedViewport<R>>>>;

/// Initialize embedded viewport by creating a child window for wgpu rendering
///
/// This creates a child window (not WebviewWindow) that uses wgpu for direct
/// GPU rendering at 60+ FPS. The child window moves with the parent automatically.
///
/// IMPORTANT: On macOS, Metal layer creation MUST happen on the main thread.
/// Tauri commands run on the async runtime, so we use run_on_main_thread().
#[tauri::command]
pub async fn viewport_init_native<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
) -> BridgeResult<bool> {
    // Check if already initialized - prevent duplicate initialization from React StrictMode
    if state.is_embedded_mode() {
        info!("Embedded viewport already initialized, skipping duplicate init");
        return Ok(true);
    }
    
    info!("Initializing embedded viewport {}x{} at ({}, {})", width, height, x, y);

    // Verify main window exists before proceeding
    let _main_window = app
        .get_webview_window("main")
        .ok_or_else(|| crate::error::BridgeError::ViewportInit("Main window not found".to_string()))?;

    // Clone references for move into closure
    let embedded_mode_flag = state.embedded_mode.clone();
    let app_clone = app.clone();

    // Use a channel to communicate the result from main thread back to async context
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

    // === CRITICAL: Run viewport initialization on main thread ===
    // On macOS, Metal layer creation MUST happen on the main thread.
    // Tauri's run_on_main_thread ensures we're on the correct thread.
    app.run_on_main_thread(move || {
        info!("Running embedded viewport init on main thread...");
        
        let result = (|| -> Result<(), String> {
            // Get main window again in this closure
            let main_window = app_clone
                .get_webview_window("main")
                .ok_or_else(|| "Main window not found".to_string())?;

            // Create embedded viewport as child of main window
            let mut viewport = EmbeddedViewport::new(
                &app_clone,
                &main_window,
                x, y,
                width, height,
            )?;

            // Start the render loop (creates wgpu surface on THIS thread)
            viewport.start_render_loop()?;

            // Store the viewport in managed state
            // We wrap in Arc<RwLock<Option<>>> so it can be accessed from commands
            let viewport_state: EmbeddedViewportState<R> = Arc::new(RwLock::new(Some(viewport)));
            app_clone.manage(viewport_state);
            
            if let Ok(mut mode) = embedded_mode_flag.write() {
                *mode = true;
            }
            info!("Embedded viewport initialized successfully on main thread");
            Ok(())
        })();

        let _ = tx.send(result);
    }).map_err(|e| crate::error::BridgeError::ViewportInit(format!("Main thread dispatch failed: {:?}", e)))?;

    // Wait for the result from main thread
    let init_result = rx.recv()
        .map_err(|e| crate::error::BridgeError::ViewportInit(format!("Failed to receive init result: {}", e)))?;

    match init_result {
        Ok(()) => {
            info!("Embedded viewport initialized successfully - 60+ FPS");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to initialize embedded viewport: {}", e);
            // Fall back to offscreen rendering
            state
                .init_renderer(width, height)
                .await
                .map_err(|e| crate::error::BridgeError::ViewportInit(e))?;
            info!("Fell back to offscreen rendering");
            Ok(false)
        }
    }
}

/// Check if embedded mode is active
#[tauri::command]
pub async fn viewport_is_native(state: State<'_, AppState>) -> BridgeResult<bool> {
    Ok(state.is_embedded_mode())
}

/// Update embedded viewport size and position
///
/// Called from frontend when the viewport area changes (panel resize, window move, etc.)
/// The x,y coordinates are CSS (logical) pixels relative to the webview content area.
#[tauri::command]
pub async fn viewport_update_bounds<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> BridgeResult<()> {
    if !state.is_embedded_mode() {
        // Also resize the offscreen renderer for fallback mode
        viewport_resize(state.clone(), width, height).await?;
        return Ok(());
    }

    // Get the main window to calculate coordinate conversion
    let main_window = match app.get_webview_window("main") {
        Some(w) => w,
        None => return Ok(()),
    };

    let scale_factor = main_window.scale_factor().unwrap_or(1.0);

    // Get parent window's inner size for coordinate conversion
    let inner_size = main_window.inner_size()
        .map_err(|e| crate::error::BridgeError::ViewportInit(e.to_string()))?;

    // Convert from logical to physical pixels for inner size
    let parent_height_logical = inner_size.height as f64 / scale_factor;

    // macOS child windows with parent() use bottom-left origin
    // Convert from top-down (CSS) to bottom-up (macOS) coordinates
    let child_x = x;
    let child_y = (parent_height_logical as i32) - y - (height as i32);

    // Get the embedded viewport from managed state and update bounds
    if let Some(viewport_state) = app.try_state::<EmbeddedViewportState<R>>() {
        if let Ok(guard) = viewport_state.read() {
            if let Some(viewport) = guard.as_ref() {
                let _ = viewport.update_bounds(child_x, child_y, width, height);
            }
        }
    }

    // Update stored size
    if let Ok(mut size) = state.viewport_size.write() {
        *size = (width, height);
    }

    Ok(())
}

/// Send orbit command to embedded viewport
#[tauri::command]
pub async fn viewport_orbit_native<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    delta_x: f32,
    delta_y: f32,
) -> BridgeResult<()> {
    if state.is_embedded_mode() {
        if let Some(viewport_state) = app.try_state::<EmbeddedViewportState<R>>() {
            if let Ok(guard) = viewport_state.read() {
                if let Some(viewport) = guard.as_ref() {
                    let _ = viewport.send(RenderMessage::Orbit { delta_x, delta_y });
                }
            }
        }
    } else {
        // Fallback to offscreen mode camera update
        viewport_orbit(state, OrbitInputDto { delta_x, delta_y }).await?;
    }
    Ok(())
}

/// Send pan command to embedded viewport
#[tauri::command]
pub async fn viewport_pan_native<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    delta_x: f32,
    delta_y: f32,
) -> BridgeResult<()> {
    if state.is_embedded_mode() {
        if let Some(viewport_state) = app.try_state::<EmbeddedViewportState<R>>() {
            if let Ok(guard) = viewport_state.read() {
                if let Some(viewport) = guard.as_ref() {
                    let _ = viewport.send(RenderMessage::Pan { delta_x, delta_y });
                }
            }
        }
    } else {
        viewport_pan(state, PanInputDto { delta_x, delta_y }).await?;
    }
    Ok(())
}

/// Send zoom command to embedded viewport
#[tauri::command]
pub async fn viewport_zoom_native<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    delta: f32,
) -> BridgeResult<()> {
    if state.is_embedded_mode() {
        if let Some(viewport_state) = app.try_state::<EmbeddedViewportState<R>>() {
            if let Ok(guard) = viewport_state.read() {
                if let Some(viewport) = guard.as_ref() {
                    let _ = viewport.send(RenderMessage::Zoom { delta });
                }
            }
        }
    } else {
        viewport_zoom(state, ZoomInputDto { delta }).await?;
    }
    Ok(())
}

/// FPS statistics DTO for frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FpsStatsDto {
    pub fps: f64,
    pub frame_time_ms: f64,
}

/// Get current FPS statistics from the embedded viewport
#[tauri::command]
pub async fn viewport_get_fps<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> BridgeResult<FpsStatsDto> {
    if state.is_embedded_mode() {
        if let Some(viewport_state) = app.try_state::<EmbeddedViewportState<R>>() {
            if let Ok(guard) = viewport_state.read() {
                if let Some(viewport) = guard.as_ref() {
                    let stats = viewport.fps_stats();
                    return Ok(FpsStatsDto {
                        fps: stats.fps,
                        frame_time_ms: stats.frame_time_ms,
                    });
                }
            }
        }
    }
    // Fallback: no FPS data available
    Ok(FpsStatsDto {
        fps: 0.0,
        frame_time_ms: 0.0,
    })
}
