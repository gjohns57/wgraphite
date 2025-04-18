// mod spectral_plot;
mod draw;
mod camera;
mod plot_window;
mod force_plot;
// mod draw;

// mod graph_plot;
mod texture;
mod plot_graphic;
mod mouse;

use gscientific::graph::petersen_graph;
use force_plot::ForcePlot;
use winit::
    event_loop::{ControlFlow, EventLoop}
;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn init() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let graph = petersen_graph(2500, 101);
    // graph.add_vertex();
    // graph.random_regularish(100, 2);
    // graph.resize(10);
    // graph.add_edge(0, 1);
    // graph.add_edge(1, 2);
    // graph.add_edge(2, 3);
    // graph.add_edge(3, 4);
    // graph.add_edge(4, 0);
    // graph.add_edge(0, 5);
    // graph.add_edge(1, 6);
    // graph.add_edge(2, 7);
    // graph.add_edge(3, 8);
    // graph.add_edge(4, 9);
    // graph.add_edge(5, 7);
    // graph.add_edge(6, 8);
    // graph.add_edge(7, 9);
    // graph.add_edge(8, 5);
    // graph.add_edge(9, 6);
    // let mut rng = rng();
    // for _i in 0..100 {
    //     let mut g = AdjacencyMatrix::new();
    //     let old_vertices_ct = graph.vertex_ct();
    //     let new_vertices_ct = (rng.next_u32() % 25 + 5) as usize;
    //     g.random(new_vertices_ct , 0.7);
    //     graph.merge(&g);
        
    //     for j in 0..(match (rng.next_u32() as usize).checked_rem(old_vertices_ct / 100) { Some(val) => val, None => 0}) {
    //         graph.add_edge(rng.next_u32() as usize % old_vertices_ct, rng.next_u32() as usize % new_vertices_ct + old_vertices_ct);
    //     }
    // }

    // let u = graph.add_vertex();
    // let v = graph.add_vertex();
    // graph.add_edge(u, v);

    let graph_plot = ForcePlot::new(graph);


    let mut app = plot_window::PlotWindow::new(graph_plot);


    event_loop.run_app(&mut app).unwrap();
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
