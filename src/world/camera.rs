use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use glam::{Mat4, Quat, Vec3};

use crate::input::InputState;

use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Default)]
pub struct Camera {
    position: Vec3,

    pitch: f32, // Up/Down, x-axis rotation
    yaw: f32,   // Left/Right, y-axis rotation

    projection: Mat4,
}

impl Camera {
    pub fn update(&mut self, _delta: f32, input: &InputState) {
        const SENSITIVITY: f32 = 0.001;
        const SPEED: f32 = 0.01;

        let (dx, dy) = input.mdelta();

        self.pitch =
            (self.pitch + (dy as f32) * SENSITIVITY).clamp(-FRAC_PI_2 + 0.1, FRAC_PI_2 - 0.1);
        self.yaw += (dx as f32) * SENSITIVITY;

        let view_dir = self.view_dir();
        let right_dir = Quat::from_rotation_y(self.yaw) * Vec3::X;

        let mut move_dir = Vec3::default();

        if input.is_key_pressed(KeyCode::KeyW) {
            move_dir += view_dir;
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            move_dir -= view_dir;
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            move_dir += right_dir;
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            move_dir -= right_dir;
        }
        if input.is_key_pressed(KeyCode::Space) {
            move_dir += Vec3::Y;
        }
        if input.is_key_pressed(KeyCode::ControlLeft) {
            move_dir -= Vec3::Y;
        }

        let speed_mul = if input.is_key_pressed(KeyCode::ShiftLeft) {
            3.0
        } else {
            1.0
        };

        self.position += move_dir.normalize_or_zero() * speed_mul * SPEED;
    }

    pub fn write_view_matrix_buffer(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        // FIXME: Write buffer only when changed
        let matrix = Mat4::look_to_lh(self.position, self.view_dir(), Vec3::Y);
        queue.write_buffer(
            buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(matrix.as_ref()),
        );
    }

    fn view_dir(&self) -> Vec3 {
        Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0) * Vec3::Z
    }

    pub fn write_projection_matrix_buffer(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer, aspect: f32) {
        let projection = Self::projection(aspect);
        queue.write_buffer(
            buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(projection.as_ref()),
        );
    }

    fn projection(aspect: f32) -> Mat4 {
        Mat4::perspective_lh(80.0f32.to_radians(), aspect, 0.1, 100.0)
    }

}
