// Wireframe shader with camera transforms
//
// Bind groups:
// - @group(0) @binding(0): CameraUniform
// - @group(1) @binding(0): ModelUniform

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_col0: vec4<f32>,
    normal_col1: vec4<f32>,
    normal_col2: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Wireframe color (dark gray for contrast)
const WIREFRAME_COLOR: vec4<f32> = vec4<f32>(0.1, 0.1, 0.1, 1.0);

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    
    // Use vertex color or wireframe color
    out.color = in.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
