use std::sync::Arc;

use instant::Instant;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowId}
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::texture;

const REDRAW_RATE: u128 = 1000 / 60;

pub trait PlotGraphic {
    fn init(&mut self, state: &PlotWindowState);
    fn tick(&mut self, delta_t: instant::Duration);
    fn input(&mut self, event: &WindowEvent, window_size: PhysicalSize<u32>) -> bool;
    // fn draw(&mut self, state: &mut PlotWindowState);
    fn update(&mut self, window_state: &mut PlotWindowState, rewrite_data: bool);
    fn render(&mut self, window_state: &mut PlotWindowState) -> Result<(), wgpu::SurfaceError>;
}


pub struct PlotWindow<T: PlotGraphic> {
    window: Option<Arc<Window>>,
    state: Option<PlotWindowState<'static>>,
    last_tick: Instant,
    plot_graphic: T,
}

impl<T: PlotGraphic> PlotWindow<T> {
    pub fn new(plot_graphic: T) -> PlotWindow<T> {
        Self {
            window: None,
            state: None,
            last_tick: Instant::now(),
            plot_graphic: plot_graphic,
        }
    }
}

impl<T: PlotGraphic> ApplicationHandler for PlotWindow<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            // let _ = window.request_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let window = Arc::new(window);

        self.window = Some(window.clone());
        self.state = Some(pollster::block_on(PlotWindowState::new(window)));
        self.plot_graphic.init(self.state.as_ref().unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();

        if self.plot_graphic.input(&event, state.size)
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
                // self.plot_graphic.update(state, false);
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();

                let now = Instant::now();
                if now.duration_since(self.last_tick).as_millis() > REDRAW_RATE {
                    self.plot_graphic.tick(now.duration_since(self.last_tick));
                    self.plot_graphic.update(state, true);
                    self.last_tick = now;

                    // state.update();
                    match self.plot_graphic.render(state) {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                            log::error!("OutOfMemory");
                            event_loop.exit();
                        }

                        // This happens when the a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub struct PlotWindowState<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: texture::Texture,
}

impl<'a> PlotWindowState<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> PlotWindowState<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                            .using_resolution(adapter.limits())
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let mut width = new_size.width;
            let mut height = new_size.height;
            // #[cfg(target_arch = "wasm32")]
            // {
            //     width = std::cmp::min(width, 1024);
            //     height = std::cmp::min(height, 1024);
            // }
            let new_new_size = winit::dpi::PhysicalSize::<u32>::new(width, height);
            self.size = new_size;
            self.config.width = new_new_size.width;
            self.config.height = new_new_size.height;

            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
        }
    }
}
