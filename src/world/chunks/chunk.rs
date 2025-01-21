use glam::{Vec3, Mat4};

use wgpu::util::DeviceExt;

pub struct NoData;
pub struct GPUData {
    v_buffer: wgpu::Buffer,
    mm_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
}

pub struct WorldChunk<D> {
    vertices: Vec<ChunkVertex>,
    position: Vec3,
    gpu_data: D,
}

impl WorldChunk<NoData> {
    pub fn generate(position: Vec3) -> Self {
        let vertices = vec![
            ChunkVertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                color: Vec3::new(1.0, 0.0, 0.0),
            },
            ChunkVertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                color: Vec3::new(0.0, 1.0, 0.0),
            },
            ChunkVertex {
                position: Vec3::new(0.0, 0.5, 0.0),
                color: Vec3::new(0.0, 0.0, 1.0),
            },
        ];
        Self {
            vertices,
            position,
            gpu_data: NoData,
        }
    }
    
    pub fn upload_to_gpu(
        self,
        gfx: &crate::GfxContext,
        chunk_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> WorldChunk<GPUData> {
        let v_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mm_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.model_matrix().as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chunk bind group"),
            layout: chunk_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mm_buffer.as_entire_binding(),
            }],
        });

        WorldChunk {
            vertices: self.vertices,
            position: self.position,
            gpu_data: GPUData {
                v_buffer,
                mm_buffer,

                bind_group,
            },
        }
    }
}

impl WorldChunk<GPUData> {
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.gpu_data.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.gpu_data.v_buffer.slice(..));
        render_pass.draw(0..self.vertices.len() as u32, 0..1);
    }
}

impl<D> WorldChunk<D> {
    fn model_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    position: Vec3,
    color: Vec3,
}

impl ChunkVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
