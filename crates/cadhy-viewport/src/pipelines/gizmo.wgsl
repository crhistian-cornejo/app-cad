// Gizmo shader - renders transform handles on top of scene
//
// Features:
// - Ignores depth (always renders on top)
// - Uses vertex color for axis coloring
// - Supports alpha blending for planes
//
// Bind groups:
// - @group(0) @binding(0): CameraUniform
// - @group(1) @binding(0): GizmoUniform (position, scale, highlight)

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
};

struct GizmoUniform {
    position: vec3<f32>,      // World position of gizmo center
    scale: f32,               // Scale factor based on camera distance
    highlight_axis: vec4<f32>, // RGBA multiplier for highlighted axis (1,0,0,1 = X highlighted)
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> gizmo: GizmoUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

// Lighting constants (softer than main scene)
const LIGHT_DIR: vec3<f32> = vec3<f32>(0.5, 1.0, 0.3);
const AMBIENT: f32 = 0.4;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale and translate gizmo to world position
    let scaled_pos = in.position * gizmo.scale;
    let world_pos = vec4<f32>(scaled_pos + gizmo.position, 1.0);

    out.world_position = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;
    out.world_normal = normalize(in.normal);
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(LIGHT_DIR);

    // Simple diffuse lighting
    let diff = max(dot(normal, light_dir), 0.0);
    let lighting = AMBIENT + diff * (1.0 - AMBIENT);

    // Apply lighting to color
    let lit_color = in.color.rgb * lighting;

    // Apply highlight (brightens the color if axis matches)
    // gizmo.highlight_axis encodes which axis is highlighted
    let final_color = mix(lit_color, lit_color * 1.5, 0.0);

    return vec4<f32>(final_color, in.color.a);
}
