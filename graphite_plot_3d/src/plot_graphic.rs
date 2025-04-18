
use std::collections::HashMap;

use wgpu::{util::DeviceExt, PipelineLayout, RenderPass};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use crate::{camera::{Camera, CameraController, CameraUniform}, plot_window::PlotWindowState, texture};

pub trait PlotGraphicElement<T> {
    fn update(&mut self, window_state: &PlotWindowState, data: &T);
    fn render(&self, graphic_state: &PlotGraphicState<T>, render_pass: &mut RenderPass);
}

pub struct PlotGraphicState<T> {
    pub render_pipeline_layout: PipelineLayout,
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_controller: CameraController,

    elements: Vec<Box<dyn PlotGraphicElement<T>>>,
}

impl<T> PlotGraphicState<T> {
    pub fn new(window_state: &PlotWindowState) -> Self {
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: window_state.config.width as f32 / window_state.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_buffer = window_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            window_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = window_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            window_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let camera_controller = CameraController::new(0.05);

        let elements = Vec::new();

        Self {
            render_pipeline_layout,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            elements,
        }
    }

    pub fn render(&self, window_state: &PlotWindowState) -> Result<(), wgpu::SurfaceError>  {
        let output = window_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = window_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &window_state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for element in &self.elements {
                element.render(self, &mut render_pass);
            }

            // println!("Rendered!");
        }

        window_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn add_element(&mut self, element: impl PlotGraphicElement<T> + 'static) {
        self.elements.push(Box::new(element));
    }

    pub fn update_data(&mut self, window_state: &PlotWindowState, data: &T) {
        for element in &mut self.elements {
            element.update(window_state, data);
        }
    }

    pub fn update(&mut self, window_state: &PlotWindowState) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera.aspect = window_state.config.width as f32 / window_state.config.height as f32;
        self.camera_uniform.update(&self.camera);
        window_state.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}