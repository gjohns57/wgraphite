use std::collections::HashSet;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

pub struct MouseTracker {
    mouse_down: HashSet<MouseButton>,
    position: PhysicalPosition<f32>,
    scroll_delta: f32,
    cursor_in_window: bool,
}

pub enum MouseEvent {
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    CursorMoved,
    CursorDragged(MouseButton),
    CursorEntered,
    CursorExited,
    WheelScrolled,
}

impl MouseTracker {
    pub fn new() -> MouseTracker {
        Self {
            mouse_down: HashSet::new(),
            position: PhysicalPosition { x: 0., y: 0. },
            scroll_delta: 0.,
            cursor_in_window: true,
        }
    }

    pub fn translate_event(&mut self, event: &WindowEvent) -> Option<MouseEvent> {
        match event {
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_down.insert(button.clone());
                    Some(MouseEvent::ButtonPressed(button.clone()))
                }
                ElementState::Released => {
                    self.mouse_down.remove(&button);
                    Some(MouseEvent::ButtonReleased(button.clone()))
                }
            },
            WindowEvent::MouseWheel { delta, .. } => { 
                match delta {
                    MouseScrollDelta::LineDelta(_v_lines, h_lines) => {
                        self.scroll_delta += h_lines;
                    },
                    MouseScrollDelta::PixelDelta(pixel_offset) => {
                        self.scroll_delta += (pixel_offset.y / 250.0) as f32;
                    }
                };
                Some(MouseEvent::WheelScrolled)
            },
            WindowEvent::CursorEntered { .. } => {
                self.cursor_in_window = true;
                Some(MouseEvent::CursorEntered)
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_in_window = false;
                Some(MouseEvent::CursorExited)
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.position = position.cast();
                
                if self.mouse_down.contains(&MouseButton::Left)  {
                    Some(MouseEvent::CursorDragged(MouseButton::Left))
                }
                else if self.mouse_down.contains(&MouseButton::Right) {
                    Some(MouseEvent::CursorDragged(MouseButton::Right))
                }
                else {
                    Some(MouseEvent::CursorMoved)
                }
            }
            _ => None,
        }
    }

    pub fn get_position(&self) -> PhysicalPosition<f32> {
        self.position
    }

    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.mouse_down.contains(&button)
    }

    pub fn consume_scroll_delta(&mut self) -> f32 {
        let tmp = self.scroll_delta;
        self.scroll_delta = 0.;
        tmp
    }

    pub fn is_cursor_in_window(&self) -> bool {
        self.cursor_in_window
    }
}