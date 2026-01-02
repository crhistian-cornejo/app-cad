//! Viewport commands: camera control, resize, render mode

use tauri::State;
use tracing::{error, info};

use cadhy_viewport::ViewMode;

use crate::dto::{CameraDto, OrbitInputDto, PanInputDto, ViewModeDto, ViewportSizeDto, ZoomInputDto};
use crate::error::BridgeResult;
use crate::state::{AppState, CameraState};

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
