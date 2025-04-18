use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::WindowEvent};
use cgmath::{EuclideanSpace, Vector2};

pub struct Camera {
    pub center: cgmath::Point2<f32>,
    pub size: PhysicalSize<f32>,

}

impl Camera {
    pub fn pan(&mut self, cursor_delta: Vector2<f32>) {
        self.center += cursor_delta
    }

    pub fn zoom(&mut self, wheel_delta: f32) {
        self.size.width *= (-0.1 * wheel_delta).exp();
        self.size.height *= (-0.1 * wheel_delta).exp();
    }

    pub fn pixel_to_camera(&self, pixel: PhysicalPosition<f32>, window_size: PhysicalSize<u32>) -> Vector2<f32> {
        let ret = self.center.to_vec() + Vector2::new((pixel.x / window_size.width as f32 - 0.5) * self.size.width, (0.5 - pixel.y / window_size.height as f32) * self.size.height);
        ret
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    center: [f32; 2],
    size: [f32; 2],
}

impl CameraUniform {
    pub fn new(window_dimension: PhysicalSize<f32>) -> Self {
        let center = [0.0, 0.0];
        let size = window_dimension.into();

        Self {
            center,
            size,
        }
    }

    pub fn update(&mut self, camera: &Camera) {

        self.center = camera.center.into();
        self.size = camera.size.into();
    }
}