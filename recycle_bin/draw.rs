use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3, VectorSpace};
use crate::{plot_graphic_2d::PlotGraphicState, plot_graphic_3d::PlotGraphicElement, plot_window::PlotWindowState};
use wgpu::Color;

pub struct Point<const D: usize, T: VectorSpace + Into<[f32; D]>> {
    position: T,
    color: Color,
    size: f32,
}

pub struct Line<const D: usize, T: VectorSpace + Into<[f32; D]>> {
    start: T,
    end: T,
    color: Color,
    width: f32,
}

fn color_to_arr(color: &Color) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}

impl<const D: usize, T: VectorSpace + Into<[f32; D]>> Point<D, T> {
    pub fn new(position: T, color: Color, size: f32) -> Self {
        Self { position, color, size }
    }
}

impl<const D: usize, T: VectorSpace + Into<[f32; D]>> Point<D, T> {
    fn get_instance(&self) -> PointInstance<D> {
        let point_instance = PointInstance {
            position: self.position.into(),
            color: color_to_arr(&self.color),
            size: self.size
        };

        point_instance
    }
}

impl<const D: usize, T: VectorSpace + Into<[f32; D]>> Line<D, T> {
    pub fn new(start: T, end: T, color: Color, width: f32) -> Self {
        Self { start, end, color, width }
    }
}

pub trait GetPoints<const D: usize> {
    type VectorType: VectorSpace + Into<[f32; D]>;

    fn get_points(&self) -> impl Iterator<Item = Point<D, Self::VectorType>>;
}


pub trait GetLines<const D: usize> {
    type VectorType: VectorSpace + Into<[f32; D]>;

    fn get_lines(&self) -> impl Iterator<Item =Line<D, Self::VectorType>>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable)]
struct PointInstance<const D: usize> {
    position: [f32; D],
    color: [f32; 3],
    size: f32
}

unsafe impl<const D: usize> Pod for PointInstance<D> {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable)]
struct LineVertex<const D: usize> {
    position: [f32; D],
    next: [f32; D],
    offset_distance: f32,
    color: [f32; 3],
}

unsafe impl<const D: usize> Pod for LineVertex<D> {}

pub struct PointElement<const D: usize> {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,

    instances: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PointVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
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

impl<const D: usize, T: GetPoints<D>> PlotGraphicElement<T> for PointElement<D> {
    fn update(&mut self, window_state: &PlotWindowState, graphic: &T) {
        let buffer = graphic.get_points()
            .map(|point| point.get_instance()).collect::<Vec<PointInstance<D>>>();

        self.instances = buffer.len();

        window_state.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(buffer.as_ref()),
        );
    }

    fn render(&self, graphic_state: &crate::plot_graphic_3d::PlotGraphicState<T>, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &graphic_state.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..(POINT_QUAD_INDEX.len() as u32),
            0,
            0..self.instances as _,
        );
    }
}

impl PointElement<2> {
    pub fn new<T: GetPoints<2>>(
        graphic_state: &PlotGraphicState<T>,
        window_state: &PlotWindowState,
    ) -> Self {
        let shader = window_state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Point Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../assets/point_shader_2d.wgsl").into()),
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
                        buffers: &[PointVertex::desc(), PointInstance2D::desc()], // 2.
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
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

        let instances = 0;

        Self {
            render_pipeline,
            instance_buffer,
            vertex_buffer,
            index_buffer,
            instances,
        }
    }
}

