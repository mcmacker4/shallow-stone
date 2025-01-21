pub mod camera;
pub mod chunks;

use crate::{
    input::InputState,
    world::{camera::Camera, chunks::Chunks},
};

use glam::{Quat, Vec3};

pub struct World {
    chunks: Chunks,
    pub camera: Camera,
}

impl World {
    pub fn new(gfx: &crate::GfxContext) -> Self {
        Self {
            chunks: Chunks::init(gfx),
            camera: Camera::default(),
        }
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
    }

    pub fn prepare_render(&mut self, gfx: &crate::GfxContext) {
        self.chunks.prepare_render(gfx);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.chunks.render(render_pass);
    }
}
