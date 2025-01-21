#![allow(unused)]

pub mod input;
pub mod render;
pub mod world;

pub mod macros;

use winit::keyboard::{Key, KeyCode, PhysicalKey};
use world::World;

use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, WindowId};

use input::InputState;
use render::GfxContext;

use pollster::block_on;

use anyhow::{Context, Result};

#[derive(Default)]
struct App {
    state: Option<InitializedApp>,
}

struct InitializedApp {
    gfx: GfxContext,
    world: World,

    mouse_grabbed: bool,

    last_update: std::time::Instant,

    pub input: InputState,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resumed");

        drop(self.state.take());

        let mut state = block_on(InitializedApp::create(event_loop))
            .expect("Could not create a render context");

        state.initialize();

        self.state.replace(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }

        if let Some(state) = &mut self.state {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    state.on_resize(size);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    state.on_keyboard_key(event, event_loop);
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                } => {
                    state.on_cursor_pos(position);
                }
                _ => (),
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(state) = &mut self.state {
                state.on_mouse_move(delta);
            }
        }
    }

    fn about_to_wait(&mut self, _loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.update();
            state.render();
            state.input.on_frame_end();
        }
    }

    fn suspended(&mut self, _loop: &ActiveEventLoop) {
        println!("Suspended");
        // Replace render context with None, dropping the current one
        self.state.take();
    }
}

impl InitializedApp {
    async fn create(evento_loop: &ActiveEventLoop) -> Result<Self> {
        let gfx = GfxContext::create(evento_loop).await?;

        let world = World::new(&gfx);

        Ok(Self {
            gfx,
            world,
            mouse_grabbed: false,
            last_update: Instant::now(),
            input: InputState::new(),
        })
    }

    pub fn initialize(&mut self) {
        //self.gfx
        //    .window
        //    .set_cursor_grab(CursorGrabMode::Locked)
        //    .unwrap();
        //self.gfx.window.set_cursor_visible(false);
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_update);

        let delta_time = duration.as_secs_f32();

        self.world.update(delta_time, &self.input);

        self.last_update = now;
    }

    pub fn render(&mut self) {
        self.gfx.render(&mut self.world);
    }

    pub fn on_keyboard_key(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        match event {
            key_press!(Escape) => event_loop.exit(),
            key_press!(KeyG) => self.grab_mouse(),
            KeyEvent {
                physical_key: PhysicalKey::Code(code),
                state,
                repeat: false,
                ..
            } => {
                self.input.on_keyboard_key(code, state);
            }
            _ => (),
        }
    }

    pub fn on_mouse_move(&mut self, delta: (f64, f64)) {
        if self.mouse_grabbed {
            self.input.on_mouse_move(delta);
        }
    }

    pub fn on_cursor_pos(&mut self, position: PhysicalPosition<f64>) {
        self.input.on_cursor_pos(position);
    }

    fn on_resize(&mut self, size: PhysicalSize<u32>) {
        self.gfx.resize(size);
        self.gfx.update_projection_matrix_buffer(&self.world.camera);
    }

    fn grab_mouse(&mut self) {
        let grab = !self.mouse_grabbed;

        let next = match grab {
            true => winit::window::CursorGrabMode::Locked,
            false => winit::window::CursorGrabMode::None,
        };

        if self.gfx.window.set_cursor_grab(next).is_ok() {
            self.gfx.window.set_cursor_visible(!grab);
            self.mouse_grabbed = grab;
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    event_loop
        .run_app(&mut app)
        .expect("Could not run event loop");
}
