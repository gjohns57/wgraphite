use instant::Duration;
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::MouseButton};
use crate::{
    draw_3d::{Color, GetLines, GetPoints, LineElement, PointElement }, graph::Graph, mouse::{MouseEvent, MouseTracker}, plot_graphic::{PlotGraphic, PlotGraphicState}, plot_window::PlotWindowState
};
use cgmath::{num_traits::Pow, InnerSpace, Transform, Vector2, Vector3, Vector4, Zero};
use rand::random;

const VERTEX_REPULSION: f32 = 1.0;
const EDGE_SPRING_STIFFNESS: f32 = 6.0;
const EDGE_EQ_LENGTH: f32 = 1.0;
const RESISTANCE: f32 = 4.0;
const CENTERING: f32 = 0.5;

struct SpectralPlotModel<T: Graph> {
    graph: T,

    positions: Vec<Vector3<f32>>,
    colors: Vec<Color>,
    sizes: Vec<f32>,
}

impl<T: Graph> SpectralPlotModel<T> {
    fn tick(&mut self, delta_t: Duration) {
       
    }
}

pub struct SpectralPlot<T: Graph> {
    model: SpectralPlotModel<T>,
    
    mouse_tracker: MouseTracker,
    drag_previous_position: Vector2<f32>,
    state: Option<PlotGraphicState<SpectralPlotModel<T>>>,
}

fn pixel_to_clip(pixel: PhysicalPosition<f32>, window_size: PhysicalSize<u32>) -> Vector2<f32> {
    Vector2::new(2.0 * pixel.x / window_size.width as f32 - 1.0, 1.0 - 2. * pixel.y / window_size.height as f32)
}

impl<T: Graph> PlotGraphic for SpectralPlot<T> {
    fn init(&mut self, state: &PlotWindowState) {
        self.state = Some(PlotGraphicState::new(state));
        let point_element = PointElement::new(self.state.as_ref().unwrap(), state);
        let line_element = LineElement::new(self.state.as_ref().unwrap(), state);
        self.state.as_mut().unwrap().add_element(point_element);
        self.state.as_mut().unwrap().add_element(line_element);
    }

    fn tick(&mut self, delta_t: instant::Duration) {
        self.model.tick(delta_t);
    }

    fn input(&mut self, event: &winit::event::WindowEvent, window_size: PhysicalSize<u32>) -> bool {
        if self.state.as_mut().unwrap().camera_controller.process_events(event) {
            return true;
        }

        match self.mouse_tracker.translate_event(event) {
            Some(mouse_event) => {
                match mouse_event {
                    MouseEvent::ButtonPressed(MouseButton::Left) => {
                        let mouse_pixel_pos = self.mouse_tracker.get_position();
                        let mouse_position = pixel_to_clip(mouse_pixel_pos, window_size);

                        self.drag_previous_position = mouse_position;
                    },
                    MouseEvent::CursorDragged => {
                        let mouse_position = pixel_to_clip(self.mouse_tracker.get_position(), window_size);
                        let delta = mouse_position - self.drag_previous_position;
                        self.drag_previous_position = mouse_position;

                        self.state.as_mut().unwrap().camera.pan(delta);
                    },
                    MouseEvent::WheelScrolled => {
                        self.state.as_mut().unwrap().camera.zoom(self.mouse_tracker.consume_scroll_delta());
                    },
                    _ => {

                    }
                }
            },
            None => {

            }
        };

        false
    }

    fn render(&mut self, window_state: &mut PlotWindowState) -> Result<(), wgpu::SurfaceError> {
        match &self.state {
            Some(state) => {
                state.render(window_state)?;
            },
            None => {
                println!("Graphic state not initialized yet!");
            }
        };

        Ok(())
    }
    
    fn update(&mut self, window_state: &mut PlotWindowState, rewrite_data: bool) {
        let state = self.state.as_mut();

        state.map(|state| {
            if rewrite_data {
                state.update_data(window_state, &self.model);
            }
            state.update(window_state);
        });
    }
}

impl<T: Graph> SpectralPlot<T> {
    pub fn new(graph: T) -> Self {
        let colors = (0..graph.vertex_ct()).map(|i| {if i % 2 == 0 { Color::rgb(0.1, 0.7, 0.1) } else { Color::rgb(0.1, 0.1, 0.7)}}).collect();
        let mut sizes= Vec::new();
        sizes.resize(graph.vertex_ct(), 0.1);
        let state = None;
        let drag_previous_position = Vector2::zero();

        let model = SpectralPlotModel {
            graph,
            colors,
            sizes,
        };

        // model.tick(Duration::new(0, 1000000));
        // for particle in &mut model.particles {
        //     particle.clear_velocity();
        // }

        Self {
            model,
            state,
            mouse_tracker: MouseTracker::new(),
            drag_previous_position,
        }
    }
}

impl<T: Graph> GetPoints for SpectralPlotModel<T> {
    fn get_points(&self) -> impl Iterator<Item = (Vector3<f32>, Color, f32)> {
        (0..self.graph.vertex_ct()).map(|i| (self.positions[i], self.colors[i], self.sizes[i]))
    }
    // fn get_points(&self) -> impl Iterator<Item = Vector3<f32>> {
    //     self.particles.iter().map(|particle| particle.position)
    // }
}

impl<T: Graph> GetLines for SpectralPlotModel<T> {
    fn get_lines(&self) -> impl Iterator<Item = (Vector3<f32>, Vector3<f32>, Color)> {
        self.graph.edges().map(|(u,v)| 
            (self.positions[u] + (self.positions[v] - self.positions[u]).normalize() * self.sizes[u],
             self.positions[v] - (self.positions[v] - self.positions[u]).normalize() * self.sizes[v],
            Color::rgba(0., 0., 0., 0.5)))
    }
}