
use bytemuck::{Pod, Zeroable};
use cgmath::Vector2;
use wgpu::util::DeviceExt;

use crate::{
    plot_graphic::{PlotGraphicElement, PlotGraphicState},
    plot_window::PlotWindowState,
    texture,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        let a = 1.0;

        Self { r, g, b, a }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LineVertex {
    position: [f32; 2],
    next: [f32; 2],
    offset_distance: f32,
    color: [f32; 3],
}

impl LineVertex {
    pub fn new(
        v: Vector2<f32>,
        next: Vector2<f32>,
        offset_distance: f32,
        color: [f32; 3],
    ) -> LineVertex {
        Self {
            position: v.into(),
            next: next.into(),
            offset_distance: offset_distance,
            color: color,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 2 * std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: (2 * std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<f32>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PointVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl PointVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PointVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const POINT_QUAD: &[PointVertex] = &[
    PointVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 0.0],
    },
    PointVertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    PointVertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 0.0],
    },
    PointVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];

const POINT_QUAD_INDEX: &[u16] = &[0, 2, 1, 1, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
struct PointInstance {
    position: [f32; 2],
    color: [f32; 3],
    size: f32,
}

impl PointInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PointInstance>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress + mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct PointElement {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,

    instances: Vec<PointInstance>,
}

pub trait GetPoints {
    fn get_points(&self) -> impl Iterator<Item = (Vector2<f32>, Color, f32)>;
}

impl<T: GetPoints> PlotGraphicElement<T> for PointElement {
    fn update(&mut self, window_state: &PlotWindowState, graphic: &T) {
        self.instances.clear();

        for (position, color, size) in graphic.get_points() {
            self.instances.push(PointInstance {
                position: position.into(),
                color: color.into(),
                size: size,
            });
        }

        // println!("{:?}", self.instances);
        window_state.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(self.instances.as_ref()),
        );
    }

    fn render(&self, graphic_state: &PlotGraphicState<T>, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &graphic_state.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..(POINT_QUAD_INDEX.len() as u32),
            0,
            0..self.instances.len() as _,
        );
    }
}

impl PointElement {
    pub fn new<T: GetPoints>(
        graphic_state: &PlotGraphicState<T>,
        window_state: &PlotWindowState,
    ) -> Self {
        let shader = window_state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Point Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../assets/point_shader.wgsl").into()),
            });

        let render_pipeline =
            window_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&graphic_state.render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"), // 1.
                        buffers: &[PointVertex::desc(), PointInstance::desc()], // 2.
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 3.
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: window_state.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    // depth_stencil: Some(wgpu::DepthStencilState {
                    //     format: texture::Texture::DEPTH_FORMAT,
                    //     depth_write_enabled: true,
                    //     depth_compare: wgpu::CompareFunction::Less,
                    //     stencil: wgpu::StencilState::default(),
                    //     bias: wgpu::DepthBiasState::default(),
                    // }), // 1.
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,                         // 2.
                        mask: !0,                         // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                    cache: None,     // 6.
                });

        let instance_buffer = window_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Point Instance Buffer"),
            size: 1 << 20,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer =
            window_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Point Vertex Buffer"),
                    contents: bytemuck::cast_slice(POINT_QUAD),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            window_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Point Index Buffer"),
                    contents: bytemuck::cast_slice(POINT_QUAD_INDEX),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let instances = Vec::new();

        Self {
            render_pipeline,
            instance_buffer,
            vertex_buffer,
            index_buffer,
            instances,
        }
    }
}

pub trait GetLines {
    fn get_lines(&self) -> impl Iterator<Item = (Vector2<f32>, Vector2<f32>, Color)>;
}

pub struct LineElement {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    vertices: Vec<LineVertex>,
    indices: Vec<u16>,
}

impl<T: GetLines> PlotGraphicElement<T> for LineElement {
    fn update(&mut self, window_state: &PlotWindowState, lines: &T) {
        let mut index = 0;
        self.vertices.clear();
        self.indices.clear();

        for (x0, x1, color) in lines.get_lines() {
            self.vertices
                .push(LineVertex::new(x0, x1, 0.02, color.into()));
            self.vertices
                .push(LineVertex::new(x0, x1, -0.02, color.into()));
            self.vertices
                .push(LineVertex::new(x1, x0, -0.02, color.into()));
            self.vertices
                .push(LineVertex::new(x1, x0, 0.02, color.into()));

            self.indices.push(index + 0);
            self.indices.push(index + 1);
            self.indices.push(index + 3);
            self.indices.push(index + 0);
            self.indices.push(index + 3);
            self.indices.push(index + 2);
            index += 4;
        }

        window_state.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertices),
        );
        window_state
            .queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
    }

    fn render(&self, graphic_state: &PlotGraphicState<T>, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline); // 2.
        render_pass.set_bind_group(0, &graphic_state.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(self.indices.len() as u32), 0, 0..1);
    }
}

impl LineElement {
    pub fn new<T: GetLines>(
        graphic_state: &PlotGraphicState<T>,
        window_state: &PlotWindowState,
    ) -> Self {
        let shader = window_state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../assets/line_shader.wgsl").into()),
            });

        let render_pipeline =
            window_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&graphic_state.render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),   // 1.
                        buffers: &[LineVertex::desc()], // 2.
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 3.
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: window_state.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    // depth_stencil: Some(wgpu::DepthStencilState {
                    //     format: texture::Texture::DEPTH_FORMAT,
                    //     depth_write_enabled: true,
                    //     depth_compare: wgpu::CompareFunction::Less,
                    //     stencil: wgpu::StencilState::default(),
                    //     bias: wgpu::DepthBiasState::default(),
                    // }), // 1.
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,                         // 2.
                        mask: !0,                         // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                    cache: None,     // 6.
                });

        let vertex_buffer = window_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Edge Vertex Buffer"),
            size: 1 << 21,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = window_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Edge Index Buffer"),
            size: 1 << 20,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertices = Vec::new();
        let indices = Vec::new();

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            indices,
            vertices,
        }
    }
}
