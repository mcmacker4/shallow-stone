mod camera;
mod chunk;

use camera::Camera;
use chunk::{ChunkVertex, LoadedChunk, UnloadedChunk, WorldChunk};

use super::input::InputState;

use glam::{Vec3, Quat};

pub struct World {
    unloaded_chunks: Vec<UnloadedChunk>,

    chunks: Vec<LoadedChunk>,
    camera: Camera,

    chunk_pipeline: wgpu::RenderPipeline,
    chunk_bind_group_layout: wgpu::BindGroupLayout,
}

impl World {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("./world.wgsl"));

        let aspect = config.width as f32 / config.height as f32;
        let camera = Camera::new(device, aspect);

        let chunk_bind_group_layout = chunk::chunk_bind_group_layout(&device);

        let chunk_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("world pipeline layout"),
                bind_group_layouts: &[camera.bind_group_layout(), &chunk_bind_group_layout],
                push_constant_ranges: &[],
            });

        let chunk_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world pipeline"),
            layout: Some(&chunk_pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[ChunkVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::render::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            unloaded_chunks: vec![
                WorldChunk::new().at_position(Vec3::new(0.0, 0.0, 2.0)),
                WorldChunk::new().at_position(Vec3::new(0.0, 0.0, -2.0)),

                WorldChunk::new().at_position(Vec3::new(2.0, 0.0, 0.0)).with_rotation(Quat::from_rotation_y(90.0f32.to_radians())),
                WorldChunk::new().at_position(Vec3::new(-2.0, 0.0, 0.0)).with_rotation(Quat::from_rotation_y(90.0f32.to_radians())),
            ],
            chunks: vec![],

            camera,

            chunk_pipeline,
            chunk_bind_group_layout,
        }
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
        for chunk in &mut self.chunks {
            chunk.update(delta);
        }
    }

    pub fn prepare_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.load_pending_chunks(device);
        self.camera.update_view_matrix(queue);
        for chunk in &self.chunks {
            chunk.prepare_render(queue);
        }
    }

    fn load_pending_chunks(&mut self, device: &wgpu::Device) {
        if let Some(unloaded) = self.unloaded_chunks.pop() {
            let loaded = unloaded.gpu_load(device, &self.chunk_bind_group_layout);
            self.chunks.push(loaded);
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.camera.render(render_pass);
        self.render_chunks(render_pass);
    }

    fn render_chunks(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.chunk_pipeline);

        for chunk in &self.chunks {
            chunk.render(render_pass);
        }
    }

    pub(crate) fn update_camera_projection(&self, queue: &wgpu::Queue, aspect: f32) {
        self.camera.update_projection_matrix(queue, aspect);
    }
}
