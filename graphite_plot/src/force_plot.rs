use instant::Duration;
use winit::{dpi::PhysicalSize, event::{ElementState, KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}};

use crate::{
    draw::{Color, GetLines, GetPoints, LineElement, PointElement},
    mouse::{MouseEvent, MouseTracker},
    plot_graphic::PlotGraphicState,
    plot_window::{PlotGraphic, PlotWindowState},
};

use gscientific::{graph::Graph, linalg::{BasicMatrix, Matrix}};

use cgmath::{EuclideanSpace, InnerSpace, Vector2, Zero, num_traits::Pow};
use rand::random;

const EDGE_SPRING_STIFFNESS: f32 = 1.0;
const EDGE_EQ_LENGTH_MULT: f32 = 2.0;
const RESISTANCE: f32 = 1.0;
const CENTERING: f32 = 0.1;
const INIT_SPACING_MULTIPLIER: f32 = 1.0;
const SPEED_LIMIT: f32 = 100.0;

#[derive(PartialEq, Eq)]
enum InteractAction {
    Drag,
    New,
    AddEdge,
}

struct ForcePlotModel<T: Graph> {
    graph: T,
    graph_distances: BasicMatrix<i32>,
    particles: Vec<PhysicsParticle>,
    colors: Vec<Color>,
    sizes: Vec<f32>,
    interacted_particle: Option<(usize, InteractAction)>,
    pause: bool,
}

impl<T: Graph> ForcePlotModel<T> {
    fn new(
        graph: T,
        particles: Vec<PhysicsParticle>,
        colors: Vec<Color>,
        sizes: Vec<f32>,
        interacted_particle: Option<(usize, InteractAction)>,
    ) -> Self {
        let graph_distances = graph.unweighted_all_pairs_distance();
        let pause = false;

        Self {
            graph,
            graph_distances,
            particles,
            colors,
            sizes,
            interacted_particle,
            pause,
        }
    }

    fn tick(&mut self, delta_t: Duration) {
        if self.pause {
            return;
        }

        for u in 0..self.particles.len() {
            self.particles[u].clear_force();
            let tmp_force = -self.particles[u].get_velocity() * RESISTANCE
                - self.particles[u].get_position() * CENTERING;
            self.particles[u].add_force(tmp_force);

            if self.interacted_particle.is_some()
                && u == self.interacted_particle.as_ref().unwrap().0  && self.interacted_particle.as_ref().unwrap().1 == InteractAction::New
            {
                continue;
            }

            for v in 0..u {
                if self.interacted_particle.is_some()
                    && v == self.interacted_particle.as_ref().unwrap().0 && self.interacted_particle.as_ref().unwrap().1 == InteractAction::New
                {
                    continue;
                }
                let graph_distance = *self.graph_distances.get(u, v);
                let eq_length = if graph_distance != -1 { EDGE_EQ_LENGTH_MULT * graph_distance as f32 } else { 20.0 * EDGE_EQ_LENGTH_MULT };
                let r = (self.particles[u].get_position() - self.particles[v].get_position())
                    .magnitude();
                let direction =
                    (self.particles[u].get_position() - self.particles[v].get_position()) / r;

                // self.particles[u].add_force(1. / (r * r) * VERTEX_REPULSION * direction);
                // self.particles[v].add_force(-1. / (r * r) * VERTEX_REPULSION * direction);

                self.particles[u].add_force(
                    -(r / eq_length).ln()
                        * EDGE_SPRING_STIFFNESS
                        * direction,
                );
                self.particles[v].add_force(
                    (r / eq_length).ln() * EDGE_SPRING_STIFFNESS * direction,
                );
            }
        }

        for u in 0..self.particles.len() {
            if self.interacted_particle.is_some()
                && u == self.interacted_particle.as_ref().unwrap().0
            {
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
        match event {
            winit::event::WindowEvent::KeyboardInput { event: KeyEvent {
                state: ElementState::Pressed,
                physical_key: PhysicalKey::Code(KeyCode::Space),
                ..
            }, .. } => {
                self.model.pause = !self.model.pause;
            },
            winit::event::WindowEvent::KeyboardInput { event: KeyEvent {
                state: ElementState::Pressed,
                physical_key: PhysicalKey::Code(KeyCode::KeyZ),
                ..
            }, .. } => {
                self.model.particles.pop();
                self.model.graph.resize(self.model.graph.vertex_ct() - 1);
                self.model.colors.pop();
                self.model.sizes.pop();

                self.model.graph_distances = self.model.graph.unweighted_all_pairs_distance();
            },
            winit::event::WindowEvent::KeyboardInput { event: KeyEvent {
                state: ElementState::Pressed,
                physical_key: PhysicalKey::Code(KeyCode::KeyC),
                ..
            }, .. } => {
                self.model.particles.clear();
                self.model.graph.resize(1);
                self.model.colors.clear();
                self.model.sizes.clear();
                self.model.particles.push(PhysicsParticle::new(Vector2::zero(), 1.0));
                self.model.sizes.push(0.1);
                self.model.colors.push(Color::rgb(0.1, 0.9, 0.1));

                self.model.graph_distances = self.model.graph.unweighted_all_pairs_distance();

            },
            _ => {}
        }

        match self.mouse_tracker.translate_event(event) {
            Some(mouse_event) => {
                match mouse_event {
                    MouseEvent::ButtonPressed(MouseButton::Left) => {
                        let mouse_position = self
                            .state
                            .as_ref()
                            .unwrap()
                            .camera
                            .pixel_to_camera(self.mouse_tracker.get_position(), window_size);
                        let mouse_position_delta =
                            self.state.as_ref().unwrap().camera.center.to_vec() - mouse_position;
                        self.drag_previous_position = mouse_position_delta;
                        self.model.interacted_particle = self
                            .get_particle_by_position(mouse_position)
                            .map(|index| (index, InteractAction::Drag))
                    }
                    MouseEvent::ButtonPressed(MouseButton::Right) => {
                        let mouse_position = self
                            .state
                            .as_ref()
                            .unwrap()
                            .camera
                            .pixel_to_camera(self.mouse_tracker.get_position(), window_size);

                        match self.get_particle_by_position(mouse_position) {
                            Some(index) => {
                                let v = self.model.graph.add_vertex();
                                self.model.graph.add_edge(index, v);
                                self.model.graph_distances =
                                    self.model.graph.unweighted_all_pairs_distance();
                                self.model
                                    .particles
                                    .push(PhysicsParticle::new(mouse_position, 1.0));
                                self.model.colors.push(Color::rgb(0.1, 0.7, 0.1));
                                self.model.sizes.push(0.1);
                                self.model.interacted_particle = Some((v, InteractAction::New));
                            }
                            None => {}
                        }
                    }
                    MouseEvent::ButtonPressed(MouseButton::Middle) => {
                        self.model.particles.pop();
                        self.model.graph.resize(self.model.graph.vertex_ct() - 1);
                        self.model.colors.pop();
                        self.model.sizes.pop();

                        self.model.graph_distances = self.model.graph.unweighted_all_pairs_distance();
                    }
                    MouseEvent::ButtonReleased(button) => {
                        match &self.model.interacted_particle {
                            Some((index, action)) => match action {
                                InteractAction::New => {
                                    let mouse_position =
                                        self.state.as_ref().unwrap().camera.pixel_to_camera(
                                            self.mouse_tracker.get_position(),
                                            window_size,
                                        );
                                    let to_merge = self.get_particle_by_position(mouse_position);
                                    match to_merge {
                                        Some(merge_index) => {
                                            let incident_index =
                                                self.model.graph.neighbors(*index).next().unwrap().0;
                                            if incident_index != merge_index {
                                                self.model.particles.pop();
                                                self.model
                                                    .graph
                                                    .resize(self.model.graph.vertex_ct() - 1);
                                                self.model.colors.pop();
                                                self.model.graph_distances =
                                                    self.model.graph.unweighted_all_pairs_distance();
                                                self.model.sizes.pop();

                                                self.model
                                                    .graph
                                                    .add_edge(incident_index, merge_index);
                                                self.model.graph_distances =
                                                    self.model.graph.unweighted_all_pairs_distance();
                                            }
                                            else {
                                                self.model.graph.remove_vertex(*index);
                                                self.model.graph.remove_vertex(incident_index);

                                                self.model.colors.remove(*index);
                                                self.model.colors.remove(incident_index);
                                                self.model.particles.remove(*index);
                                                self.model.particles.remove(incident_index);
                                                self.model.sizes.remove(*index);
                                                self.model.sizes.remove(incident_index);

                                                self.model.graph_distances = self.model.graph.unweighted_all_pairs_distance();
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                _ => {}
                            },
                            None => {}
                        }
                        self.model.interacted_particle = None;
                    }
                    MouseEvent::CursorDragged(button) => {
                        let mouse_position = self
                            .state
                            .as_ref()
                            .unwrap()
                            .camera
                            .pixel_to_camera(self.mouse_tracker.get_position(), window_size);
                        match &self.model.interacted_particle {
                            Some((index, action)) => {
                                self.model.particles[*index].position = mouse_position;

                                // TODO
                                // let mouse_clip = pixel_to_clip(self.mouse_tracker.get_position(), window_size);
                                // let particle_clip = self.state.as_ref().unwrap().camera.world_to_clip(self.model.particles[index].position);
                                // let new_clip = Vector3::new(mouse_clip.x, mouse_clip.y, particle_clip.z);
                                // let new_world = self.state.as_ref().unwrap().camera.clip_to_world(new_clip);
                                // self.model.particles[index].position = new_world;
                            }
                            None => match button {
                                MouseButton::Left => {
                                    let mouse_position_delta =
                                        self.state.as_ref().unwrap().camera.center.to_vec()
                                            - mouse_position;
                                    let delta = mouse_position_delta - self.drag_previous_position;
                                    self.drag_previous_position = mouse_position_delta;

                                    self.state.as_mut().unwrap().camera.pan(delta);
                                }
                                _ => {}
                            },
                        }
                    }
                    MouseEvent::WheelScrolled => {
                        self.state
                            .as_mut()
                            .unwrap()
                            .camera
                            .zoom(self.mouse_tracker.consume_scroll_delta());
                    }
                    _ => {}
                }
            }
            None => {}
        };

        false
    }

    fn render(&mut self, window_state: &mut PlotWindowState) -> Result<(), wgpu::SurfaceError> {
        match &self.state {
            Some(state) => {
                state.render(window_state)?;
            }
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

impl<T: Graph> ForcePlot<T> {
    fn get_particle_by_position(&self, position: Vector2<f32>) -> Option<usize> {
        let interacted_particle = match self.model.interacted_particle {
            Some((index, _)) => index,
            None => usize::MAX,
        };
        for (i, particle) in self.model.particles.iter().enumerate() {
            if i == interacted_particle {
                continue;
            }
            if (particle.position - position).magnitude() < self.model.sizes[i] {
                return Some(i);
            }
        }

        None
    }
}

// fn logistic(x: f32) -> f32 {
//     1.0 / (1.0 + (-x).exp())
// }

pub struct PhysicsParticle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    force: Vector2<f32>,
    mass: f32,
}

impl PhysicsParticle {
    pub fn new(position: Vector2<f32>, mass: f32) -> PhysicsParticle {
        Self {
            position: position,
            velocity: Vector2::zero(),
            force: Vector2::zero(),
            mass: mass,
        }
    }

    pub fn add_force(&mut self, force: Vector2<f32>) {
        self.force += force;
    }

    pub fn clear_force(&mut self) {
        self.force = Vector2::zero();
    }

    pub fn clear_velocity(&mut self) {
        self.velocity = Vector2::zero();
    }

    pub fn step(&mut self, delta_t: f32) {
        let weight = 0.0;

        let acc = self.force / self.mass;
        let new_velocity = self.velocity + acc * delta_t;
        self.position += (weight * self.velocity + (1. - weight) * new_velocity) * delta_t; // + 0.5 * acc * delta_t * delta_t;
        self.velocity = new_velocity;
        if self.velocity.magnitude() > SPEED_LIMIT {
            self.velocity = self.velocity.normalize() * SPEED_LIMIT;
        }
    }

    pub fn get_position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn get_velocity(&self) -> Vector2<f32> {
        self.velocity
    }
}

impl<T: Graph> ForcePlot<T> {
    pub fn new(graph: T) -> Self {
        let particles = (0..graph.vertex_ct())
            .map(|_i| {
                PhysicsParticle::new(
                    Vector2::new(random::<f32>().cos(), random::<f32>().sin())
                        * (graph.vertex_ct() as f32).pow(1. / 2.)
                        * INIT_SPACING_MULTIPLIER,
                    1.0,
                )
            })
            .collect::<Vec<PhysicsParticle>>();
        let colors = (0..graph.vertex_ct())
            .map(|i| {
                    Color::rgb(0.1, 0.7, 0.1)
            })
            .collect();
        let mut sizes = Vec::new();
        sizes.resize(graph.vertex_ct(), 0.1);
        let state = None;
        let interacted_particle = None;
        let drag_previous_position = Vector2::zero();

        let mut model = ForcePlotModel::new(graph, particles, colors, sizes, interacted_particle);

        // for _ in 0..10000 {
        //     model.tick(Duration::new(0, 10000000));
        // }
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

impl<T: Graph> GetPoints for ForcePlotModel<T> {
    fn get_points(&self) -> impl Iterator<Item = (Vector2<f32>, Color, f32)> {
        (0..self.graph.vertex_ct())
            .map(|i| (self.particles[i].position, self.colors[i], self.sizes[i]))
    }
    // fn get_points(&self) -> impl Iterator<Item = Vector3<f32>> {
    //     self.particles.iter().map(|particle| particle.position)
    // }
}

impl<T: Graph> GetLines for ForcePlotModel<T> {
    fn get_lines(&self) -> impl Iterator<Item = (Vector2<f32>, Vector2<f32>, Color)> {
        self.graph.edges().map(|(u, v, _)| {
            (
                self.particles[u].position
                    + (self.particles[v].position - self.particles[u].position).normalize() * 0.1,
                self.particles[v].position
                    - (self.particles[v].position - self.particles[u].position).normalize() * 0.1,
                Color::rgba(0., 0., 0., 0.5),
            )
        })
    }
}
