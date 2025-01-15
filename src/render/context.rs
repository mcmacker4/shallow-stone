use anyhow::{Context, Result};

use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use wgpu::{
    include_wgsl, Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RenderPipeline, RenderPipelineDescriptor, Surface, SurfaceConfiguration,
};

use super::texture::Texture;

use std::sync::Arc;

use crate::world::World;

pub struct GfxContext {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,

    pub config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    depth_texture: super::texture::Texture,

    pub device: Device,
    pub queue: Queue,
}

impl GfxContext {
    pub async fn create(event_loop: &ActiveEventLoop) -> Result<GfxContext> {
        let window = Arc::new(Self::create_window(&event_loop).context("Create window")?);

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

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

        Ok(Self {
            window,
            surface,
            config,
            size,
            depth_texture,
            device,
            queue,
        })
    }

    fn create_window(event_loop: &ActiveEventLoop) -> Result<Window> {
        let window_attributes = WindowAttributes::default();
            //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        Ok(event_loop
            .create_window(window_attributes)
            .context("Create WINIT window")?)
    }

    fn create_instance() -> Instance {
        wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    async fn create_device(adapter: &Adapter) -> Result<(Device, Queue)> {
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
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
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth texture");
        }
    }

    pub fn render(&self, world: &mut World) -> Result<()> {
        world.prepare_render(&self.device, &self.queue);

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

    fn render_pass(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, world: &World) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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

        world.render(&mut pass);
    }
}
