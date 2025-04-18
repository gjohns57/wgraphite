struct CameraUniform {
    center: vec2<f32>,
    size: vec2<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(2) position: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) size: f32,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    let position_projected = vec2<f32>((instance.position.x - camera.center.x) / camera.size.x * 2.0, (instance.position.y - camera.center.y) / camera.size.y * 2.0);
    let model_position_projected = vec2<f32>(model.position.x * instance.size / camera.size.x * 2.0, model.position.y * instance.size / camera.size.y * 2.0);
    out.clip_position = vec4<f32>(position_projected + model_position_projected, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r2 = (in.tex_coords.x - 0.5) * (in.tex_coords.x - 0.5) + (in.tex_coords.y - 0.5) * (in.tex_coords.y - 0.5);
    let outline_thickness = 0.1;

    if r2 <= (0.5 - outline_thickness) * (0.5 - outline_thickness) {
        return vec4<f32>(in.color, 1.0);
    }
    else if r2 <= 0.25 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    else {
        discard;
    }
}
 