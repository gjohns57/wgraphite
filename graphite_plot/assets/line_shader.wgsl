// Vertex shader
struct CameraUniform {
    center: vec2<f32>,
    size: vec2<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) current: vec2<f32>,
    @location(1) next: vec2<f32>,
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


    var current_projected: vec2<f32> = vec2<f32>((model.current.x - camera.center.x) / camera.size.x * 2.0, (model.current.y - camera.center.y) / camera.size.y * 2.0);

    // var next_projected: vec2<f32> = vec2<f32>((model.next.x - camera.center.x) / camera.size.x * 2.0, (model.next.y - camera.center.y) / camera.size.y * 2.0);

    let dir = normalize(model.next - model.current);
    let normal = vec2<f32>(-dir.y, dir.x) * model.offset_distance;
    let normal_projected = vec2<f32>(normal.x / camera.size.x * 2.0, normal.y / camera.size.y * 2.0);


    out.color = model.color;
    out.clip_position = vec4<f32>(current_projected + normal_projected, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 0.5);
}