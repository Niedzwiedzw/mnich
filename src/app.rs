use {
    crate::{
        line_label,
        wgpu_ext::{global_context::Gpu, surface::render_attachment_surface::RenderAttachmentSurface},
    },
    anyhow::Context,
    futures::TryFutureExt,
    next_tuple::NextTuple,
    std::{future::ready, num::NonZeroU32, sync::Arc},
    tap::prelude::*,
    tracing::{debug, info, instrument},
    wgpu::{Color, CurrentSurfaceTexture},
    winit::{
        application::ApplicationHandler,
        event::{self, KeyEvent, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::{self, Window, WindowAttributes},
    },
};

impl App {
    pub async fn new(window_attributes: WindowAttributes, event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        event_loop
            .create_window(window_attributes.clone())
            .with_context(|| format!("could not create a window with window attributes {window_attributes:#?}"))
            .tap(|window| {
                info!(?window, "window created");
            })
            .map(Arc::new)
            .pipe(ready)
            .and_then(async |window| {
                Gpu::new()
                    .map_ok(|gpu| gpu.init_global())
                    .and_then(async |gpu| RenderAttachmentSurface::new(window.clone()).map(|surface| (gpu, surface)))
                    .await
                    .map(|t| t.next_tuple(window))
            })
            .map_ok(|(gpu, surface, window)| Self { gpu, surface, window })
            .await
    }
}

#[derive(Debug)]
pub struct App {
    pub window: Arc<Window>,
    pub surface: RenderAttachmentSurface,
    gpu: &'static Gpu,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    #[instrument(skip(self, event_loop, _window_id))]
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: window::WindowId, event: WindowEvent) {
        debug!("new event");

        (match event {
            WindowEvent::Resized(physical_size) => physical_size
                .width
                .pipe(NonZeroU32::new)
                .zip(physical_size.height.pipe(NonZeroU32::new))
                .map(|(width, height)| {
                    self.surface.resize(width, height);
                })
                .pipe(|_| Ok(())),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit().pipe(Ok),
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();
                (match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(surface_texture) | CurrentSurfaceTexture::Suboptimal(surface_texture) => Ok(Some(surface_texture)),
                    CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Validation => {
                        Ok(None)
                    }
                    CurrentSurfaceTexture::Lost => Err(anyhow::anyhow!("device lost")),
                })
                .context("reading surface")
                .and_then(|surface| match surface {
                    Some(frame) => frame
                        .texture
                        .create_view(&wgpu::wgt::TextureViewDescriptor {
                            label: line_label!(),
                            ..Default::default()
                        })
                        .pipe(|view| {
                            Gpu::global()
                                .with_command_encoder("render pass", |encoder| {
                                    encoder
                                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: line_label!(),
                                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                                view: &view,
                                                depth_slice: None,
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(Color::RED),
                                                    store: wgpu::StoreOp::Store,
                                                },
                                            })],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                            multiview_mask: None,
                                        })
                                        .pipe(|_p| ())
                                        .pipe(Ok)
                                })
                                .map(|()| self.window.pre_present_notify())
                                .map(|()| Gpu::global().queue.present(frame))
                        }),
                    None => Ok(()),
                })
            }
            _ => Ok(()),
        })
        .pipe(|r| match r {
            Ok(()) => {}
            Err(error) => tracing::error!("{error:#?}"),
        })
    }
}
