use wgpu::util::DeviceExt;

use glam::{Quat, Vec3};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    position: Vec3,
    color: Vec3,
}

pub struct GPUData {
    v_buffer: wgpu::Buffer,
    mm_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
}

pub struct NoData;

pub struct WorldChunk<D> {
    vertices: Vec<ChunkVertex>,

    position: Vec3,
    rotation: Quat,
    scale: Vec3,

    gpu_data: D,
}

pub type UnloadedChunk = WorldChunk<NoData>;
pub type LoadedChunk = WorldChunk<GPUData>;

impl WorldChunk<NoData> {
    pub fn new() -> Self {
        Self {
            vertices: vec![
                ChunkVertex {
                    position: [-1.0, -1.0, 0.0].into(),
                    color: [1.0, 0.0, 0.0].into(),
                },
                ChunkVertex {
                    position: [1.0, -1.0, 0.0].into(),
                    color: [0.0, 1.0, 0.0].into(),
                },
                ChunkVertex {
                    position: [0.0, 1.0, 0.0].into(),
                    color: [0.0, 0.0, 1.0].into(),
                },
            ],

            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::default(),
            scale: Vec3::new(0.5, 0.5, 0.5),

            gpu_data: NoData,
        }
    }

    pub fn gpu_load(
        self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> WorldChunk<GPUData> {
        let v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mm_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.model_matrix().as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chunk bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mm_buffer.as_entire_binding(),
            }],
        });

        WorldChunk {
            vertices: self.vertices,

            position: self.position,
            rotation: self.rotation,
            scale: self.scale,

            gpu_data: GPUData {
                v_buffer,
                mm_buffer,

                bind_group,
            },
        }
    }
}

impl WorldChunk<GPUData> {
    pub fn prepare_render(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.gpu_data.mm_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(self.model_matrix().as_ref()),
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.gpu_data.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.gpu_data.v_buffer.slice(..));
        render_pass.draw(0..self.vertices.len() as u32, 0..1);
    }
}

impl<T> WorldChunk<T> {
    pub fn model_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn at_position(self, position: Vec3) -> Self {
        Self {
            position,
            ..self
        }
    }

    pub fn with_rotation(self, rotation: Quat) -> Self {
        Self {
            rotation,
            ..self
        }
    }

    pub fn update(&mut self, delta: f32) {
        //let angle = std::f32::consts::PI * delta;
        //self.rotation = self.rotation * Quat::from_rotation_y(angle);
    }
}

// Defines the layout of uniform/texture bindings for rendering a chunk
// like the chunk's model matrix
pub fn chunk_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("chunk bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

impl ChunkVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
