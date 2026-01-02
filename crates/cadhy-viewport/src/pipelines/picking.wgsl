// Picking shader - outputs unique color per object

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>, // Used as pick color here
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pick_color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.pick_color = in.color; // Pass pick color through
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.pick_color;
}
