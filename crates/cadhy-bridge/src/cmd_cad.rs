//! CAD commands: Boolean operations, primitives, transformations
//!
//! These commands bridge the React UI to the OpenCASCADE kernel via cadhy-cad.
//! All operations work on B-Rep shapes stored in AppState::shape_store.

use cadhy_cad::{MeshData as CadMeshData, Operations, Primitives, Shape};
use tauri::State;
use uuid::Uuid;

use crate::dto::{PrimitiveParams, TransformDto};
use crate::error::{BridgeError, BridgeResult};
use crate::state::AppState;

/// Boolean operation type
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOp {
    /// Union: combine two shapes into one
    Fuse,
    /// Difference: subtract second shape from first
    Cut,
    /// Intersection: keep only the common volume
    Common,
}

/// Result of a CAD operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CadOperationResult {
    pub id: Uuid,
    pub name: String,
    pub success: bool,
    pub message: Option<String>,
}

// ============================================================================
// Primitive Creation with B-Rep
// ============================================================================

/// Create a CAD primitive and add it to the scene with B-Rep storage
#[tauri::command]
pub async fn cad_create_primitive(
    state: State<'_, AppState>,
    name: String,
    primitive: PrimitiveParams,
    transform: Option<TransformDto>,
) -> BridgeResult<CadOperationResult> {
    use cadhy_viewport::{SceneObject, Transform};
    use glam::{Quat, Vec3};

    // Create B-Rep shape based on primitive type
    let shape = match &primitive {
        PrimitiveParams::Box { width, height, depth } => {
            Primitives::make_box(*width, *height, *depth)
                .map_err(|e| BridgeError::CadOperation(e.to_string()))?
        }
        PrimitiveParams::Sphere { radius } => Primitives::make_sphere(*radius)
            .map_err(|e| BridgeError::CadOperation(e.to_string()))?,
        PrimitiveParams::Cylinder { radius, height } => {
            Primitives::make_cylinder(*radius, *height)
                .map_err(|e| BridgeError::CadOperation(e.to_string()))?
        }
        PrimitiveParams::Cone { radius, height } => {
            // A cone has base_radius and top_radius=0 (pointed tip)
            Primitives::make_cone(*radius, 0.0, *height)
                .map_err(|e| BridgeError::CadOperation(e.to_string()))?
        }
        PrimitiveParams::Torus {
            major_radius,
            minor_radius,
        } => Primitives::make_torus(*major_radius, *minor_radius)
            .map_err(|e| BridgeError::CadOperation(e.to_string()))?,
        PrimitiveParams::Plane { width, height } => {
            // Planes are handled differently - just create a face
            Primitives::make_box(*width, *height, 0.001)
                .map_err(|e| BridgeError::CadOperation(e.to_string()))?
        }
    };

    // Tessellate to get mesh for rendering
    let mesh_result = shape
        .tessellate(0.1)
        .map_err(|e| BridgeError::CadOperation(format!("Tessellation failed: {}", e)))?;

    // Check if renderer is initialized
    let renderer_opt = state
        .renderer
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let renderer = renderer_opt
        .as_ref()
        .ok_or_else(|| BridgeError::ViewportInit("Renderer not initialized".to_string()))?;

    // Convert tessellation to MeshData
    let mesh_data = cad_mesh_to_viewport_mesh(&mesh_result)?;

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
        name: name.clone(),
        transform: scene_transform,
        mesh: Some(gpu_mesh),
        visible: true,
        selected: false,
        pick_color,
    };

    // Store the B-Rep shape
    state.store_shape(id, shape);

    // Add to scene
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    scene.add(object);
    state.mark_dirty();

    Ok(CadOperationResult {
        id,
        name,
        success: true,
        message: None,
    })
}

// ============================================================================
// Boolean Operations
// ============================================================================

/// Perform a boolean operation on two scene objects
#[tauri::command]
pub async fn cad_boolean(
    state: State<'_, AppState>,
    object_a: Uuid,
    object_b: Uuid,
    operation: BooleanOp,
    result_name: Option<String>,
    keep_originals: Option<bool>,
) -> BridgeResult<CadOperationResult> {
    use cadhy_viewport::{SceneObject, Transform};
    use glam::{Quat, Vec3};

    // Get shapes from store
    let shape_a = state
        .get_shape(object_a)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_a))?;
    let shape_b = state
        .get_shape(object_b)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_b))?;

    // Perform boolean operation
    let result_shape = match operation {
        BooleanOp::Fuse => Operations::fuse(&shape_a, &shape_b),
        BooleanOp::Cut => Operations::cut(&shape_a, &shape_b),
        BooleanOp::Common => Operations::common(&shape_a, &shape_b),
    }
    .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    // Simplify the result to clean up boolean artifacts
    let simplified = Operations::simplify(&result_shape, true, true)
        .unwrap_or_else(|_| result_shape.clone());

    // Tessellate for rendering
    let mesh_result = simplified
        .tessellate(0.1)
        .map_err(|e| BridgeError::CadOperation(format!("Tessellation failed: {}", e)))?;

    // Check if renderer is initialized
    let renderer_opt = state
        .renderer
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let renderer = renderer_opt
        .as_ref()
        .ok_or_else(|| BridgeError::ViewportInit("Renderer not initialized".to_string()))?;

    // Convert tessellation to MeshData
    let mesh_data = cad_mesh_to_viewport_mesh(&mesh_result)?;

    // Upload mesh to GPU
    let gpu_mesh = mesh_data.upload(renderer.device());

    let id = Uuid::new_v4();
    let op_name = match operation {
        BooleanOp::Fuse => "Fuse",
        BooleanOp::Cut => "Cut",
        BooleanOp::Common => "Common",
    };
    let name = result_name.unwrap_or_else(|| format!("Boolean_{}", op_name));

    let bytes = id.as_bytes();
    let pick_color = [bytes[0], bytes[1], bytes[2], 255];

    let object = SceneObject {
        id,
        name: name.clone(),
        transform: Transform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        mesh: Some(gpu_mesh),
        visible: true,
        selected: false,
        pick_color,
    };

    // Store the result shape
    state.store_shape(id, simplified);

    // Remove original objects if requested
    let keep = keep_originals.unwrap_or(false);
    if !keep {
        let mut scene = state
            .scene
            .write()
            .map_err(|e| BridgeError::StateLock(e.to_string()))?;

        scene.remove(object_a);
        scene.remove(object_b);
        state.remove_shape(object_a);
        state.remove_shape(object_b);

        scene.add(object);
    } else {
        let mut scene = state
            .scene
            .write()
            .map_err(|e| BridgeError::StateLock(e.to_string()))?;
        scene.add(object);
    }

    state.mark_dirty();

    Ok(CadOperationResult {
        id,
        name,
        success: true,
        message: Some(format!("Boolean {} completed", op_name)),
    })
}

/// Fuse (union) two objects - convenience wrapper
#[tauri::command]
pub async fn cad_fuse(
    state: State<'_, AppState>,
    object_a: Uuid,
    object_b: Uuid,
    result_name: Option<String>,
    keep_originals: Option<bool>,
) -> BridgeResult<CadOperationResult> {
    cad_boolean(
        state,
        object_a,
        object_b,
        BooleanOp::Fuse,
        result_name,
        keep_originals,
    )
    .await
}

/// Cut (subtract) object_b from object_a - convenience wrapper
#[tauri::command]
pub async fn cad_cut(
    state: State<'_, AppState>,
    object_a: Uuid,
    object_b: Uuid,
    result_name: Option<String>,
    keep_originals: Option<bool>,
) -> BridgeResult<CadOperationResult> {
    cad_boolean(
        state,
        object_a,
        object_b,
        BooleanOp::Cut,
        result_name,
        keep_originals,
    )
    .await
}

/// Common (intersection) of two objects - convenience wrapper
#[tauri::command]
pub async fn cad_common(
    state: State<'_, AppState>,
    object_a: Uuid,
    object_b: Uuid,
    result_name: Option<String>,
    keep_originals: Option<bool>,
) -> BridgeResult<CadOperationResult> {
    cad_boolean(
        state,
        object_a,
        object_b,
        BooleanOp::Common,
        result_name,
        keep_originals,
    )
    .await
}

// ============================================================================
// Fillet and Chamfer
// ============================================================================

/// Apply fillet to all edges of an object
#[tauri::command]
pub async fn cad_fillet(
    state: State<'_, AppState>,
    object_id: Uuid,
    radius: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let filleted = Operations::fillet(&shape, radius)
        .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, filleted).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Fillet".to_string(),
        success: true,
        message: Some(format!("Applied fillet radius {:.2}", radius)),
    })
}

/// Apply chamfer to all edges of an object
#[tauri::command]
pub async fn cad_chamfer(
    state: State<'_, AppState>,
    object_id: Uuid,
    distance: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let chamfered = Operations::chamfer(&shape, distance)
        .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, chamfered).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Chamfer".to_string(),
        success: true,
        message: Some(format!("Applied chamfer distance {:.2}", distance)),
    })
}

// ============================================================================
// Shell and Offset
// ============================================================================

/// Create a hollow shell from a solid
#[tauri::command]
pub async fn cad_shell(
    state: State<'_, AppState>,
    object_id: Uuid,
    thickness: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let shelled = Operations::shell(&shape, thickness)
        .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, shelled).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Shell".to_string(),
        success: true,
        message: Some(format!("Applied shell thickness {:.2}", thickness)),
    })
}

/// Offset a solid inward or outward
#[tauri::command]
pub async fn cad_offset(
    state: State<'_, AppState>,
    object_id: Uuid,
    offset: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let offset_shape = Operations::offset(&shape, offset)
        .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, offset_shape).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Offset".to_string(),
        success: true,
        message: Some(format!("Applied offset {:.2}", offset)),
    })
}

// ============================================================================
// Transform Operations
// ============================================================================

/// Translate a shape (modifies B-Rep, not just transform)
#[tauri::command]
pub async fn cad_translate(
    state: State<'_, AppState>,
    object_id: Uuid,
    dx: f64,
    dy: f64,
    dz: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let translated = Operations::translate(&shape, dx, dy, dz)
        .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, translated).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Translate".to_string(),
        success: true,
        message: Some(format!("Translated by ({:.2}, {:.2}, {:.2})", dx, dy, dz)),
    })
}

/// Rotate a shape around an axis
#[tauri::command]
pub async fn cad_rotate(
    state: State<'_, AppState>,
    object_id: Uuid,
    origin: [f64; 3],
    axis: [f64; 3],
    angle_degrees: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let angle_radians = angle_degrees.to_radians();
    let rotated = Operations::rotate(
        &shape,
        origin[0],
        origin[1],
        origin[2],
        axis[0],
        axis[1],
        axis[2],
        angle_radians,
    )
    .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, rotated).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Rotate".to_string(),
        success: true,
        message: Some(format!("Rotated by {:.1} degrees", angle_degrees)),
    })
}

/// Mirror a shape across a plane
#[tauri::command]
pub async fn cad_mirror(
    state: State<'_, AppState>,
    object_id: Uuid,
    origin: [f64; 3],
    normal: [f64; 3],
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let mirrored = Operations::mirror(
        &shape,
        origin[0],
        origin[1],
        origin[2],
        normal[0],
        normal[1],
        normal[2],
    )
    .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    // Create a new object for the mirrored copy
    update_object_shape(&state, object_id, mirrored).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Mirror".to_string(),
        success: true,
        message: Some("Created mirror copy".to_string()),
    })
}

/// Scale a shape uniformly
#[tauri::command]
pub async fn cad_scale(
    state: State<'_, AppState>,
    object_id: Uuid,
    center: [f64; 3],
    factor: f64,
) -> BridgeResult<CadOperationResult> {
    let shape = state
        .get_shape(object_id)
        .ok_or_else(|| BridgeError::ObjectNotFound(object_id))?;

    let scaled =
        Operations::scale(&shape, center[0], center[1], center[2], factor)
            .map_err(|e| BridgeError::CadOperation(e.to_string()))?;

    update_object_shape(&state, object_id, scaled).await?;

    Ok(CadOperationResult {
        id: object_id,
        name: "Scale".to_string(),
        success: true,
        message: Some(format!("Scaled by factor {:.2}", factor)),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Update an existing object's shape and mesh
async fn update_object_shape(
    state: &State<'_, AppState>,
    object_id: Uuid,
    new_shape: Shape,
) -> BridgeResult<()> {
    // Tessellate the new shape
    let mesh_result = new_shape
        .tessellate(0.1)
        .map_err(|e| BridgeError::CadOperation(format!("Tessellation failed: {}", e)))?;

    // Get renderer
    let renderer_opt = state
        .renderer
        .read()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let renderer = renderer_opt
        .as_ref()
        .ok_or_else(|| BridgeError::ViewportInit("Renderer not initialized".to_string()))?;

    // Convert tessellation to MeshData
    let mesh_data = cad_mesh_to_viewport_mesh(&mesh_result)?;

    // Upload new mesh to GPU
    let gpu_mesh = mesh_data.upload(renderer.device());

    // Update the scene object
    let mut scene = state
        .scene
        .write()
        .map_err(|e| BridgeError::StateLock(e.to_string()))?;

    let obj = scene
        .get_mut(object_id)
        .ok_or(BridgeError::ObjectNotFound(object_id))?;

    obj.mesh = Some(gpu_mesh);

    // Update shape store
    state.store_shape(object_id, new_shape);
    state.mark_dirty();

    Ok(())
}

/// Convert cadhy-cad MeshData to viewport MeshData
fn cad_mesh_to_viewport_mesh(cad_mesh: &CadMeshData) -> BridgeResult<cadhy_viewport::MeshData> {
    use cadhy_viewport::{MeshData, Vertex};

    if cad_mesh.vertices.is_empty() || cad_mesh.indices.is_empty() {
        return Err(BridgeError::CadOperation(
            "Tessellation produced empty mesh".to_string(),
        ));
    }

    // Default color for CAD objects
    let color = [0.7, 0.7, 0.7, 1.0];

    // Convert vertices: cadhy-cad MeshData has Vec<Vertex3> for positions and normals
    let mut vertices = Vec::with_capacity(cad_mesh.vertices.len());

    for (i, vert) in cad_mesh.vertices.iter().enumerate() {
        let position = [vert.x as f32, vert.y as f32, vert.z as f32];

        let normal = if i < cad_mesh.normals.len() {
            let n = &cad_mesh.normals[i];
            [n.x as f32, n.y as f32, n.z as f32]
        } else {
            [0.0, 1.0, 0.0] // Default up normal
        };

        vertices.push(Vertex {
            position,
            normal,
            color,
        });
    }

    // Indices are already u32, wrap in Some for MeshData structure
    let indices = Some(cad_mesh.indices.clone());

    Ok(MeshData { vertices, indices })
}
