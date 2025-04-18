use winit::{dpi::PhysicalSize, event::MouseButton};

use crate::{
    draw::{GetLines, GetPoints, Line, Point }, mouse::{MouseEvent, MouseTracker}, plot_graphic_2d::PlotGraphicState, plot_window::{PlotWindowState, PlotGraphic}
};


pub trait GraphLayoutScheme<const D: usize>: GetLines<D> + GetPoints<D> {
    fn input(&mut self, event: &winit::event::WindowEvent, window_size: PhysicalSize<u32>) -> bool;
    fn tick(&mut self, delta_t: instant::Duration) {}
}

pub struct GraphPlot<const D: usize, T: GraphLayoutScheme<D>> {
    model: T,
    state: Option<PlotGraphicState<T>>,
}

impl<const D: usize, T: GraphLayoutScheme<D>> PlotGraphic for GraphPlot<D, T> {
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
        if self.input(event, window_size) {
            return true;
        }
        
        match self.state {
            Some(state) => state.camera.input(event, window_size),
            None => {}
        }
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