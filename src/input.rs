use winit::event::ElementState;
use winit::keyboard::KeyCode;
use winit::dpi::PhysicalPosition;

use std::collections::HashSet;

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    mouse_delta: (f64, f64),
    mouse_position: PhysicalPosition<f64>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn on_keyboard_key(&mut self, code: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed_keys.insert(code),
            ElementState::Released => self.pressed_keys.remove(&code)
        };
    }

    pub fn on_mouse_move(&mut self, delta: (f64, f64)) {
        let (dx, dy) = self.mouse_delta;
        self.mouse_delta = (dx + delta.0, dy + delta.1);
    }

    pub fn on_cursor_pos(&mut self, pos: PhysicalPosition<f64>) {
        self.mouse_position = pos;
    }

    pub fn on_frame_end(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn is_key_pressed(&self, code: KeyCode) -> bool {
        self.pressed_keys.contains(&code)
    }

    pub fn mdelta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn mpos(&self) -> PhysicalPosition<f64> {
        self.mouse_position
    }
}


