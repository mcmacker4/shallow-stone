use anyhow::{Context, Result};

use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use wgpu::{include_wgsl, util::DeviceExt};

use std::sync::Arc;

use crate::{
    render::texture::Texture,
    world::{camera::Camera, World},
};

pub struct GfxContext {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,

    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub global_shader_bindings: GlobalShaderBindings,

    depth_texture: super::texture::Texture,
}

impl GfxContext {
    pub async fn create(event_loop: &ActiveEventLoop) -> Result<GfxContext> {
        let window = Arc::new(Self::create_window(event_loop).context("Create window")?);

        let size = window.inner_size();

        let instance = Self::create_instance();
        let surface = instance
            .create_surface(window.clone())
            .context("Create WGPU surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("Could not find compatible adapter");

        dbg!(&adapter.get_info());

        let (device, queue) = Self::create_device(&adapter).await?;

        dbg!(device.limits());

        let surface_caps = surface.get_capabilities(&adapter);

        dbg!(&surface_caps);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        dbg!(&config);

        surface.configure(&device, &config);

        let global_shader_bindings = GlobalShaderBindings::init(&device);
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

        Ok(Self {
            window,
            surface,

            config,
            size,

            device,
            queue,

            global_shader_bindings,

            depth_texture,
        })
    }

    fn create_window(event_loop: &ActiveEventLoop) -> Result<Window> {
        let window_attributes =
            WindowAttributes::default().with_inner_size(PhysicalSize::new(1280, 720));
        //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        event_loop
            .create_window(window_attributes)
            .context("Create WINIT window")
    }

    fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    async fn create_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    ..Default::default()
                },
                None,
            )
            .await
            .context("Create WGPU device")?;
        Ok((device, queue))
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth texture");
        }
    }

    pub fn render(&self, world: &mut World) -> Result<()> {
        world
            .camera
            .write_view_matrix_buffer(&self.queue, &self.global_shader_bindings.view_matrix_buffer);

        world.prepare_render(self);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        self.render_pass(&view, &mut encoder, world);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_pass(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.3,
                        g: 0.6,
                        b: 0.9,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_bind_group(0, &self.global_shader_bindings.group, &[]);

        world.render(&mut pass);
    }

    pub fn update_projection_matrix_buffer(&self, camera: &Camera) {
        let aspect = self.window_aspect_ratio();
        camera.write_projection_matrix_buffer(
            &self.queue,
            &self.global_shader_bindings.proj_matrix_buffer,
            aspect,
        );
    }

    fn window_aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }
}

pub struct GlobalShaderBindings {
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,

    pub view_matrix_buffer: wgpu::Buffer,
    pub proj_matrix_buffer: wgpu::Buffer,
}

impl GlobalShaderBindings {
    pub fn init(device: &wgpu::Device) -> Self {
        let identity: &[u8] = bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref());

        let view_matrix_buffer =
            Self::create_uniform_buffer(device, identity, Some("view matrix buffer"));
        let proj_matrix_buffer =
            Self::create_uniform_buffer(device, identity, Some("projection matrix buffer"));

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: proj_matrix_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            layout,
            group,

            view_matrix_buffer,
            proj_matrix_buffer,
        }
    }

    fn create_uniform_buffer(
        device: &wgpu::Device,
        init_data: &[u8],
        label: Option<&'static str>,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: init_data,
        })
    }
}
