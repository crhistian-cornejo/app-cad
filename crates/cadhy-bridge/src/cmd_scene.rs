//! Scene commands: object management, selection, transforms

use tauri::State;
use uuid::Uuid;

use crate::dto::{ObjectType, PrimitiveParams, SceneObjectDto, SelectionDto, TransformDto};
use crate::error::{BridgeError, BridgeResult};
use crate::state::AppState;

/// Get all objects in the scene (for outliner)
#[tauri::command]
pub async fn scene_get_objects(state: State<'_, AppState>) -> BridgeResult<Vec<SceneObjectDto>> {
    let scene = state
        .scene
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let objects: Vec<SceneObjectDto> = scene
        .objects()
        .map(|obj| SceneObjectDto {
            id: obj.id,
            name: obj.name.clone(),
            visible: obj.visible,
            selected: obj.selected,
            object_type: ObjectType::Mesh, // TODO: Store actual type
        })
        .collect();

    Ok(objects)
}

/// Add an object to the scene
#[tauri::command]
pub async fn scene_add_object(
    state: State<'_, AppState>,
    name: String,
    transform: Option<TransformDto>,
) -> BridgeResult<Uuid> {
    use cadhy_viewport::{SceneObject, Transform};
    use glam::{Quat, Vec3};

    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let id = Uuid::new_v4();

    let transform = transform.unwrap_or_default();
    let scene_transform = Transform {
        position: Vec3::from_array(transform.position),
        rotation: Quat::from_array(transform.rotation),
        scale: Vec3::from_array(transform.scale),
    };

    let object = SceneObject {
        id,
        name,
        transform: scene_transform,
        mesh: None,
        visible: true,
        selected: false,
        pick_color: [0, 0, 0, 255], // Will be assigned by scene
    };

    scene.add(object);
    state.mark_dirty();

    Ok(id)
}

/// Remove an object from the scene
#[tauri::command]
pub async fn scene_remove_object(state: State<'_, AppState>, id: Uuid) -> BridgeResult<()> {
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    if scene.remove(id).is_none() {
        return Err(BridgeError::ObjectNotFound(id));
    }

    state.mark_dirty();
    Ok(())
}

/// Select objects by ID
#[tauri::command]
pub async fn scene_select(
    state: State<'_, AppState>,
    ids: Vec<Uuid>,
    extend: bool,
) -> BridgeResult<SelectionDto> {
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    // Clear previous selection unless extending
    if !extend {
        scene.deselect_all();
    }

    // Select new objects
    for id in &ids {
        scene.select(*id, true);
    }

    let selected = scene.selected();
    state.mark_dirty();

    Ok(SelectionDto {
        count: selected.len(),
        ids: selected,
    })
}

/// Deselect all objects
#[tauri::command]
pub async fn scene_deselect_all(state: State<'_, AppState>) -> BridgeResult<()> {
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    scene.deselect_all();
    state.mark_dirty();

    Ok(())
}

/// Get current selection
#[tauri::command]
pub async fn scene_get_selection(state: State<'_, AppState>) -> BridgeResult<SelectionDto> {
    let scene = state
        .scene
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let selected = scene.selected();

    Ok(SelectionDto {
        count: selected.len(),
        ids: selected,
    })
}

/// Set transform for an object
#[tauri::command]
pub async fn scene_set_transform(
    state: State<'_, AppState>,
    id: Uuid,
    transform: TransformDto,
) -> BridgeResult<()> {
    use cadhy_viewport::Transform;
    use glam::{Quat, Vec3};

    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let object = scene.get_mut(id).ok_or(BridgeError::ObjectNotFound(id))?;

    object.transform = Transform {
        position: Vec3::from_array(transform.position),
        rotation: Quat::from_array(transform.rotation),
        scale: Vec3::from_array(transform.scale),
    };

    state.mark_dirty();
    Ok(())
}

/// Get transform for an object
#[tauri::command]
pub async fn scene_get_transform(
    state: State<'_, AppState>,
    id: Uuid,
) -> BridgeResult<TransformDto> {
    let scene = state
        .scene
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let object = scene.get(id).ok_or(BridgeError::ObjectNotFound(id))?;

    Ok(TransformDto {
        position: object.transform.position.to_array(),
        rotation: object.transform.rotation.to_array(),
        scale: object.transform.scale.to_array(),
    })
}

/// Add a primitive object (cube, sphere, etc.) to the scene with mesh
#[tauri::command]
pub async fn scene_add_primitive(
    state: State<'_, AppState>,
    name: String,
    primitive: PrimitiveParams,
    transform: Option<TransformDto>,
) -> BridgeResult<Uuid> {
    use cadhy_viewport::{MeshData, SceneObject, Transform};
    use glam::{Quat, Vec3};

    // Check if renderer is initialized (we need it for GPU mesh)
    let renderer_opt = state
        .renderer
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let renderer = renderer_opt
        .as_ref()
        .ok_or_else(|| BridgeError::ViewportInit("Renderer not initialized".to_string()))?;

    // Create mesh data based on primitive type
    let color = [0.7, 0.7, 0.7, 1.0];
    let mesh_data = match primitive {
        PrimitiveParams::Box { width, height, depth } => {
            // Use the largest dimension for uniform cube, actual box should be scaled
            let size = width.max(height).max(depth) as f32;
            MeshData::cube(size, color)
        }
        PrimitiveParams::Sphere { radius } => {
            MeshData::sphere(radius as f32, 32, 24, [0.5, 0.5, 0.8, 1.0])
        }
        PrimitiveParams::Cylinder { radius, height } => {
            MeshData::cylinder(radius as f32, height as f32, 32, [0.5, 0.8, 0.5, 1.0])
        }
        PrimitiveParams::Cone { radius, height } => {
            MeshData::cone(radius as f32, height as f32, 32, [0.8, 0.5, 0.5, 1.0])
        }
        PrimitiveParams::Torus { major_radius, minor_radius } => {
            MeshData::torus(major_radius as f32, minor_radius as f32, 32, 16, [0.8, 0.8, 0.5, 1.0])
        }
        PrimitiveParams::Plane { width, height } => {
            MeshData::plane(width as f32, height as f32, [0.6, 0.6, 0.6, 1.0])
        }
    };

    // Upload mesh to GPU
    let gpu_mesh = mesh_data.upload(renderer.device());

    let id = Uuid::new_v4();
    let transform_data = transform.unwrap_or_default();
    let scene_transform = Transform {
        position: Vec3::from_array(transform_data.position),
        rotation: Quat::from_array(transform_data.rotation),
        scale: Vec3::from_array(transform_data.scale),
    };

    // Generate pick color from UUID
    let bytes = id.as_bytes();
    let pick_color = [bytes[0], bytes[1], bytes[2], 255];

    let object = SceneObject {
        id,
        name,
        transform: scene_transform,
        mesh: Some(gpu_mesh),
        visible: true,
        selected: false,
        pick_color,
    };

    // Add to scene
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    scene.add(object);
    state.mark_dirty();

    Ok(id)
}

/// Add a default cube to the scene (convenience function)
#[tauri::command]
pub async fn scene_add_cube(
    state: State<'_, AppState>,
    name: Option<String>,
    size: Option<f32>,
    transform: Option<TransformDto>,
) -> BridgeResult<Uuid> {
    let cube_name = name.unwrap_or_else(|| "Cube".to_string());
    let cube_size = size.unwrap_or(1.0) as f64;
    
    scene_add_primitive(
        state,
        cube_name,
        PrimitiveParams::Box {
            width: cube_size,
            height: cube_size,
            depth: cube_size,
        },
        transform,
    )
    .await
}
