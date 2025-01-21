use chunk::ChunkVertex;

pub mod chunk;

use chunk::{WorldChunk, NoData, GPUData};

pub struct Chunks {
    chunks: Vec<WorldChunk<GPUData>>,
    pending_chunks: Vec<WorldChunk<NoData>>,

    pipeline: wgpu::RenderPipeline,

    bind_group_layout: wgpu::BindGroupLayout,
}

impl Chunks {
    pub fn init(gfx: &crate::GfxContext) -> Self {
        let shader = gfx.device.create_shader_module(wgpu::include_wgsl!("./chunk.wgsl"));

        let bind_group_layout = gfx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chunk bind group layout"),
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
                }
            ],
        });

        let pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("world pipeline layout"),
                    bind_group_layouts: &[
                        &gfx.global_shader_bindings.layout,
                        &bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline = gfx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world pipeline"),
            layout: Some(&pipeline_layout),
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
                    format: gfx.config.format,
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
            chunks: vec![],
            pending_chunks: vec![
                WorldChunk::generate(glam::Vec3::new(0.0, 0.0, 2.0)),
            ],

            pipeline,

            bind_group_layout,
        }
    }

    pub fn prepare_render(&mut self, gfx: &crate::GfxContext) {
        self.load_pending_chunks(gfx);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        for chunk in &self.chunks {
            chunk.render(render_pass);
        }
    }

    fn load_pending_chunks(&mut self, gfx: &crate::GfxContext) {
        if let Some(unloaded) = self.pending_chunks.pop() {
            let loaded = unloaded.upload_to_gpu(gfx, &self.bind_group_layout);
            self.chunks.push(loaded);
        }
    }
}
