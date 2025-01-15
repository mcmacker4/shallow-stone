use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use glam::{Mat4, Quat, Vec3};

use crate::input::InputState;

use std::f32::consts::PI;

pub struct Camera {
    position: Vec3,

    pitch: f32, // Up/Down, x-axis rotation
    yaw: f32,   // Left/Right, y-axis rotation

    projection: Mat4,

    view_mat_buffer: wgpu::Buffer,
    proj_mat_buffer: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let projection = Self::projection(aspect);

        let mut view_mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projection matrix buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(projection.as_ref()),
        });

        let mut proj_mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projection matrix buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(projection.as_ref()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_mat_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: proj_mat_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            projection,
            view_mat_buffer,
            proj_mat_buffer,

            position: Vec3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,

            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&mut self, _delta: f32, input: &InputState) {
        const SENSITIVITY: f32 = 0.001;
        const SPEED: f32 = 0.01;

        const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
        const WORLD_FRONT: Vec3 = Vec3::new(0.0, 0.0, 1.0);
        const WORLD_RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);

        let (dx, dy) = input.mdelta();

        self.pitch = (self.pitch + (dy as f32) * SENSITIVITY).clamp(-PI, PI);
        self.yaw += (dx as f32) * SENSITIVITY;

        let view_dir = Quat::from_rotation_y(self.yaw)
            * Quat::from_rotation_x(self.pitch)
            * WORLD_FRONT;

        let right_dir = Quat::from_rotation_y(self.yaw) * WORLD_RIGHT;

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
            move_dir += WORLD_UP;
        }
        if input.is_key_pressed(KeyCode::ControlLeft) {
            move_dir -= WORLD_UP;
        }

        //println!(
        //    "Pos[{:?}] PY[{}, {}] Move[{:?}]",
        //    self.position, self.pitch, self.yaw, move_dir
        //);
        
        let speed_mul = if input.is_key_pressed(KeyCode::ShiftLeft) { 3.0 } else { 1.0 };

        self.position += move_dir.normalize_or_zero() * speed_mul * SPEED;
    }

    pub fn update_view_matrix(&self, queue: &wgpu::Queue) {
        // FIXME: Write buffer only when changed
        let matrix = Mat4::from_euler(glam::EulerRot::XYZ, -self.pitch, -self.yaw, 0.0)
            * Mat4::from_translation(-self.position);

        queue.write_buffer(
            &self.view_mat_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(matrix.as_ref()),
        );
    }

    pub fn update_projection_matrix(&self, queue: &wgpu::Queue, aspect: f32) {
        let projection = Self::projection(aspect);
        queue.write_buffer(
            &self.proj_mat_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(projection.as_ref()),
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }

    fn projection(aspect: f32) -> Mat4 {
        Mat4::perspective_lh(80.0f32.to_radians(), aspect, 0.1, 100.0)
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}
