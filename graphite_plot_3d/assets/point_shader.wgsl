struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale_proj: mat4x4<f32>,
    aspect: f32,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(2) position: vec3<f32>,
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
    out.clip_position = camera.view_proj * vec4<f32>(instance.position, 1.0) + camera.scale_proj * vec4<f32>(model.position * instance.size, 0.0, 0.0);
    out.color = instance.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.tex_coords.x - 0.5) * (in.tex_coords.x - 0.5) + (in.tex_coords.y - 0.5) * (in.tex_coords.y - 0.5) <= 0.25 {
        return vec4<f32>(in.color * (0.5 + in.tex_coords.y * 0.5), 1.0);
    }
    else {
        discard;
    }
}
 