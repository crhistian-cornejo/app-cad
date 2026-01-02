// Grid shader with distance-based fading
//
// Bind groups:
// - @group(0) @binding(0): CameraUniform

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fade grid based on distance from camera
    let dist_from_camera = length(camera.view_pos.xz - in.world_pos.xz);
    let fade = 1.0 - smoothstep(10.0, 50.0, dist_from_camera);
    
    // Also fade based on distance from origin
    let dist_from_origin = length(in.world_pos.xz);
    let origin_fade = 1.0 - smoothstep(20.0, 100.0, dist_from_origin);
    
    let final_fade = fade * origin_fade;
    
    return vec4<f32>(in.color.rgb, in.color.a * final_fade);
}
