use {
    crate::wgpu_ext::global_context::Gpu,
    anyhow::{Context, Result},
    std::sync::Arc,
    tap::{Tap, TapFallible},
    winit::window::Window,
};

#[derive(Debug, derive_more::Deref)]
pub struct ConfiguredSurface {
    #[deref]
    surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    configuration: wgpu::SurfaceConfiguration,
}

impl ConfiguredSurface {
    pub fn new<F>(window: Arc<Window>, configuration: F) -> Result<Self>
    where
        F: FnOnce(&Gpu, &wgpu::Surface<'_>) -> Result<wgpu::SurfaceConfiguration>,
    {
        let gpu = Gpu::global();
        gpu.instance
            .create_surface(window)
            .context("creating surface")
            .and_then(|surface| configuration(gpu, &surface).map(|configuration| (surface, configuration)))
            .map(|(surface, configuration)| (surface.tap(|s| s.configure(&gpu.device, &configuration)), configuration))
            .map(|(surface, configuration)| Self { surface, configuration })
    }
}

pub mod render_attachment_surface {
    use {
        anyhow::{Context, Result},
        itertools::Itertools,
        std::sync::Arc,
        tap::prelude::*,
        winit::window::Window,
    };

    #[derive(Debug, derive_more::Deref)]
    pub struct RenderAttachmentSurface(super::ConfiguredSurface);

    impl RenderAttachmentSurface {
        pub fn new(window: Arc<Window>) -> Result<Self> {
            let size = window.inner_size();
            super::ConfiguredSurface::new(window.clone(), |gpu, surface| {
                surface.get_capabilities(&gpu.adapter).pipe(|caps| {
                    caps.formats
                        .iter()
                        .find_or_first(|t| t.is_srgb())
                        .context("no valid format found")
                        .copied()
                        .and_then(|format| {
                            wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format,
                                color_space: wgpu::SurfaceColorSpace::Srgb,
                                width: size.width,
                                height: size.height,
                                present_mode: *caps.present_modes.first().context("no present mode")?,
                                alpha_mode: *caps.alpha_modes.first().context("no alpha mode")?,
                                desired_maximum_frame_latency: 2,
                                view_formats: vec![],
                            }
                            .pipe(Ok)
                        })
                })
            })
            .map(Self)
        }
    }
}
