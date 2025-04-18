use instant::Duration;
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::MouseButton};

use crate::{
    draw::{Color, GetLines, GetPoints, LineElement, PointElement }, mouse::{MouseEvent, MouseTracker}, plot_graphic::PlotGraphicState, plot_window::{PlotWindowState, PlotGraphic}
};

use gscientific::graph::Graph;
use cgmath::{num_traits::Pow, InnerSpace, Transform, Vector2, Vector3, Vector4, Zero};
use rand::random;

const VERTEX_REPULSION: f32 = 3.0;
const EDGE_SPRING_STIFFNESS: f32 = 6.0;
const EDGE_EQ_LENGTH: f32 = 1.0;
const RESISTANCE: f32 = 0.5;
const CENTERING: f32 = 0.05;
const SPEED_LIMIT: f32 = 200.0;

struct ForcePlotModel<T: Graph> {
    graph: T,
    particles: Vec<PhysicsParticle>,
    colors: Vec<Color>,
    sizes: Vec<f32>,
    interacted_particle: Option<usize>,
}

impl<T: Graph> ForcePlotModel<T> {
    fn tick(&mut self, delta_t: Duration) {
        for u in 0..self.particles.len() {
            self.particles[u].clear_force();
            let tmp_force = -self.particles[u].get_velocity() * RESISTANCE
                - self.particles[u].get_position() * CENTERING;
            self.particles[u].add_force(tmp_force);

            for v in 0..u {
                let r = (self.particles[u].get_position() - self.particles[v].get_position())
                    .magnitude();
                let direction =
                    (self.particles[u].get_position() - self.particles[v].get_position()) / r;

                self.particles[u].add_force(1. / (r * r) * VERTEX_REPULSION * direction);
                self.particles[v].add_force(-1. / (r * r) * VERTEX_REPULSION * direction);

                if self.graph.adjacent(u, v) {
                    self.particles[u]
                        .add_force((EDGE_EQ_LENGTH - r) * EDGE_SPRING_STIFFNESS * direction);
                    self.particles[v]
                        .add_force(-(EDGE_EQ_LENGTH - r) * EDGE_SPRING_STIFFNESS * direction);
                }
            }
        }

        for u in 0..self.particles.len() {
            if self.interacted_particle.is_some() && u == self.interacted_particle.unwrap() {
                continue;
            }
            self.particles[u].step(delta_t.as_secs_f32());
        }
    }
}

pub struct ForcePlot<T: Graph> {
    model: ForcePlotModel<T>,
    
    mouse_tracker: MouseTracker,
    drag_previous_position: Vector2<f32>,
    state: Option<PlotGraphicState<ForcePlotModel<T>>>,
}

fn pixel_to_clip(pixel: PhysicalPosition<f32>, window_size: PhysicalSize<u32>) -> Vector2<f32> {
    Vector2::new(2.0 * pixel.x / window_size.width as f32 - 1.0, 1.0 - 2. * pixel.y / window_size.height as f32)
}

impl<T: Graph> PlotGraphic for ForcePlot<T> {
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

                        match self.get_particle_index_by_clip_position(mouse_position) {
                            Some(index) => {
                                self.model.interacted_particle = Some(index);
                            },
                            None => {
                                self.drag_previous_position = mouse_position;
                            }
                        }
                    },
                    MouseEvent::ButtonReleased(_) => {
                        self.model.interacted_particle = None;
                    },
                    MouseEvent::CursorDragged(_) => {
                        match self.model.interacted_particle {
                            Some(index) => {
                                let mouse_clip = pixel_to_clip(self.mouse_tracker.get_position(), window_size);
                                let particle_clip = self.state.as_ref().unwrap().camera.world_to_clip(self.model.particles[index].position);
                                let new_clip = Vector3::new(mouse_clip.x, mouse_clip.y, particle_clip.z);
                                let new_world = self.state.as_ref().unwrap().camera.clip_to_world(new_clip);
                                self.model.particles[index].position = new_world;
                            },
                            None => {
                                let mouse_position = pixel_to_clip(self.mouse_tracker.get_position(), window_size);
                                let delta = mouse_position - self.drag_previous_position;
                                self.drag_previous_position = mouse_position;

                                self.state.as_mut().unwrap().camera.pan(delta);
                            }
                        }
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

// fn logistic(x: f32) -> f32 {
//     1.0 / (1.0 + (-x).exp())
// }

pub struct PhysicsParticle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    force: Vector3<f32>,
    mass: f32,
}


impl PhysicsParticle {
    pub fn new(position: Vector3<f32>, mass: f32) -> PhysicsParticle {

        Self {
            position: position,
            velocity: Vector3::zero(),
            force: Vector3::zero(),
            mass: mass,
        }
    }

    pub fn add_force(&mut self, force: Vector3<f32>) {
        self.force += force;
    }

    pub fn clear_force(&mut self) {
        self.force = Vector3::zero();
    }

    pub fn clear_velocity(&mut self) {
        self.velocity = Vector3::zero();
    }

    pub fn step(&mut self, delta_t: f32) {
        self.velocity += self.force / self.mass * delta_t;
        let vel_mag = self.velocity.magnitude();

        if vel_mag > SPEED_LIMIT {
            self.velocity = self.velocity / vel_mag * SPEED_LIMIT;
        }
        
        self.position += self.velocity * delta_t;
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn get_velocity(&self) -> Vector3<f32> {
        self.velocity
    }
}


impl<T: Graph> ForcePlot<T> {
    pub fn new(graph: T) -> Self {
        let particles = (0..graph.vertex_ct())
            .map(|_i| {
                PhysicsParticle::new(Vector3::new(random(), random(), random()).normalize() * (graph.vertex_ct() as f32).pow(1. / 3.), 1.0)
            })
            .collect::<Vec<PhysicsParticle>>();
        let colors = (0..graph.vertex_ct()).map(|_i| {Color::rgb(0.1, 0.7, 0.1)}).collect();
        let mut sizes= Vec::new();
        sizes.resize(graph.vertex_ct(), 0.1);
        let state = None;
        let interacted_particle = None;
        let drag_previous_position = Vector2::zero();

        let mut model = ForcePlotModel {
            graph,
            colors,
            sizes,
            interacted_particle,
            particles,
        };

        Self {
            model,
            state,
            mouse_tracker: MouseTracker::new(),
            drag_previous_position,
        }
    }

    fn get_particle_index_by_clip_position(&self, cursor_position: Vector2<f32>) -> Option<usize> {
        let projection = self.state.as_ref().unwrap().camera.build_view_projection_matrix();
        let inverse_projection = projection.inverse_transform().unwrap();
        let mut min_depth = f32::MAX;
        let mut index = None;

        for i in 0..self.model.particles.len() {
            let particle_world = self.model.particles[i].position;
            let mut particle_clip = projection * Vector4::new(particle_world.x, particle_world.y, particle_world.z, 1.0);
            particle_clip /= particle_clip.w;
            let cursor_clip = Vector4::new(cursor_position.x, cursor_position.y, particle_clip.z, 1.0);
            let cursor_world_r4 = inverse_projection * cursor_clip;
            let cursor_world = Vector3::new(cursor_world_r4.x, cursor_world_r4.y, cursor_world_r4.z) / cursor_world_r4.w;


            let distance = (particle_world - cursor_world).magnitude();

            if distance < 0.1 && particle_clip.z < min_depth {
                min_depth = particle_clip.z;
                index = Some(i);
            }
        }

        index
    }
}

impl<T: Graph> GetPoints for ForcePlotModel<T> {
    fn get_points(&self) -> impl Iterator<Item = (Vector3<f32>, Color, f32)> {
        (0..self.graph.vertex_ct()).map(|i| (self.particles[i].position, self.colors[i], self.sizes[i]))
    }
    // fn get_points(&self) -> impl Iterator<Item = Vector3<f32>> {
    //     self.particles.iter().map(|particle| particle.position)
    // }
}

impl<T: Graph> GetLines for ForcePlotModel<T> {
    fn get_lines(&self) -> impl Iterator<Item = (Vector3<f32>, Vector3<f32>, Color)> {
        self.graph.edges().map(|(u,v, _)| 
            (self.particles[u].position + (self.particles[v].position - self.particles[u].position).normalize() * 0.1,
             self.particles[v].position - (self.particles[v].position - self.particles[u].position).normalize() * 0.1,
            Color::rgba(0., 0., 0., 0.5)))
    }
}