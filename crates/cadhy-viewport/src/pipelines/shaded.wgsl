// Shaded vertex/fragment shader with Blinn-Phong lighting
//
// Bind groups:
// - @group(0) @binding(0): CameraUniform
// - @group(1) @binding(0): ModelUniform

// Camera uniform - view-projection matrix and camera position
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
};

// Model uniform - model matrix and normal matrix
struct ModelUniform {
    model: mat4x4<f32>,
    // Normal matrix stored as 3 vec4s for alignment
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
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

// Lighting constants
const LIGHT_DIR: vec3<f32> = vec3<f32>(0.5, 1.0, 0.3);
const LIGHT_COLOR: vec3<f32> = vec3<f32>(1.0, 0.98, 0.95);
const AMBIENT: f32 = 0.15;
const SPECULAR_STRENGTH: f32 = 0.3;
const SHININESS: f32 = 32.0;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    
    // Transform to clip space
    out.clip_position = camera.view_proj * world_pos;
    
    // Transform normal using normal matrix
    let normal_matrix = mat3x3<f32>(
        model.normal_col0.xyz,
        model.normal_col1.xyz,
        model.normal_col2.xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    
    out.color = in.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(LIGHT_DIR);
    let view_dir = normalize(camera.view_pos - in.world_position);
    
    // Ambient
    let ambient = AMBIENT * LIGHT_COLOR;
    
    // Diffuse (Lambert)
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * LIGHT_COLOR;
    
    // Specular (Blinn-Phong)
    let half_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), SHININESS);
    let specular = SPECULAR_STRENGTH * spec * LIGHT_COLOR;
    
    // Combine lighting
    let lighting = ambient + diffuse + specular;
    let final_color = in.color.rgb * lighting;
    
    // Gamma correction (linear -> sRGB)
    let gamma_corrected = pow(final_color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma_corrected, in.color.a);
}
