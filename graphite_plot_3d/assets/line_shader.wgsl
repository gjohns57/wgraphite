// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale_proj: mat4x4<f32>,
    aspect: f32,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) current: vec3<f32>,
    @location(1) next: vec3<f32>,
    @location(2) offset_distance: f32,
    @location(3) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let current_projected = camera.view_proj * vec4<f32>(model.current, 1.0);
    var current_screen: vec2<f32> = vec2<f32>(current_projected.x, current_projected.y) / current_projected.w;

    let next_projected = camera.view_proj * vec4<f32>(model.next, 1.0);
    var next_screen: vec2<f32> = vec2<f32>(next_projected.x, next_projected.y) / next_projected.w;

    let dir = normalize(next_screen - current_screen);
    var normal: vec2<f32> = vec2<f32>(-dir.y, dir.x);

    let offset = vec4<f32>(normal * model.offset_distance, 0.0, 0.0);
    out.color = model.color;
    out.clip_position = (camera.view_proj * vec4<f32>(model.current, 1.0)) + offset;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 0.5);
}