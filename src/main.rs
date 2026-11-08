#![allow(clippy::unit_arg, reason = "I like it")]

use {
    anyhow::{Context, Result},
    winit::{event_loop::EventLoop, window::WindowAttributes},
};

mod logging;

mod utils {
    /// Exactly like [std::option::Option::zip], but for [std::result::Result]
    #[extension_traits::extension(pub trait ResultZipExt)]
    impl<T, E> std::result::Result<T, E> {
        /// Exactly like [std::option::Option::zip], but for [std::result::Result]
        fn zip<O>(self, other: std::result::Result<O, E>) -> std::result::Result<(T, O), E> {
            self.and_then(|one| other.map(|other| (one, other)))
        }
    }
}

pub mod wgpu_ext;

mod app {
    use {
        tap::prelude::*,
        tracing::{error, info},
        winit::{application::ApplicationHandler, window::WindowAttributes},
    };

    mod init_window {
        use {
            tracing::warn,
            winit::{application::ApplicationHandler, window::WindowAttributes},
        };

        #[derive(Debug)]
        pub struct InitWindow(pub WindowAttributes);

        impl ApplicationHandler for InitWindow {
            fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
                unreachable!("InitWindow mode should not be used directly");
            }

            fn window_event(
                &mut self,
                _event_loop: &winit::event_loop::ActiveEventLoop,
                _window_id: winit::window::WindowId,
                event: winit::event::WindowEvent,
            ) {
                warn!(?event, "dropping event - window still initializng");
            }
        }
    }

    mod running {
        use {
            crate::{
                line_label,
                wgpu_ext::{
                    global_context::Gpu,
                    surface::{ConfiguredSurface, render_attachment_surface::RenderAttachmentSurface},
                },
            },
            anyhow::Context,
            futures::TryFutureExt,
            next_tuple::NextTuple,
            std::{future::ready, sync::Arc},
            tap::prelude::*,
            tracing::info,
            winit::{
                application::ApplicationHandler,
                window::{Window, WindowAttributes},
            },
        };

        #[derive(Debug)]
        pub struct RunningApp {
            pub window: Arc<Window>,
            pub surface: RenderAttachmentSurface,
            gpu: &'static Gpu,
        }

        impl RunningApp {
            pub async fn new(window_attributes: WindowAttributes, event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
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

        impl ApplicationHandler for RunningApp {
            fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
                unreachable!("InitWindow mode should not be used directly");
            }

            fn window_event(
                &mut self,
                _event_loop: &winit::event_loop::ActiveEventLoop,
                _window_id: winit::window::WindowId,
                event: winit::event::WindowEvent,
            ) {
                info!(?event, "new event");
            }
        }
    }

    #[expect(clippy::large_enum_variant, reason = "It is only initialized once and it's better if it's on the stack")]
    #[derive(Debug)]
    pub enum AppMode {
        InitWindow(init_window::InitWindow),
        Running(running::RunningApp),
    }

    impl AppMode {
        pub fn new(attributes: WindowAttributes) -> Self {
            Self::InitWindow(init_window::InitWindow(attributes))
        }
    }

    macro_rules! map_variant {
        ($self:expr, [$($variant:ident),*], |$inner:ident| $map:expr) => {
            match $self {
                $(Self::$variant ($inner) => {
                    $map
                }),*
            }
        };
        ($self:expr, |$inner:ident| $map:expr) => {
            map_variant!($self, [InitWindow, Running], |$inner| $map)
        };
    }

    impl ApplicationHandler for AppMode {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            match self {
                AppMode::InitWindow(init_window::InitWindow(window_attributes)) => running::RunningApp::new(window_attributes.clone(), event_loop)
                    .pipe(futures::executor::block_on)
                    .map(Self::Running),
                AppMode::Running(running) => running
                    .resumed(event_loop)
                    .pipe(|()| Err(anyhow::anyhow!("state already transitioned"))),
            }
            .pipe(|updated| match updated {
                Ok(updated) => {
                    info!("mode updated {updated:?}");
                    *self = updated;
                }
                Err(reason) => error!("{reason:#}\n\ncould not update the mode"),
            })
        }

        fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: winit::event::WindowEvent) {
            map_variant!(self, |inner| inner.window_event(event_loop, window_id, event));
        }

        fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
            map_variant!(self, |inner| inner.new_events(event_loop, cause));
        }

        fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
            map_variant!(self, |inner| inner.user_event(event_loop, event));
        }

        fn device_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
            map_variant!(self, |inner| inner.device_event(event_loop, device_id, event));
        }

        fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            map_variant!(self, |inner| inner.about_to_wait(event_loop))
        }

        fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            map_variant!(self, |inner| inner.suspended(event_loop))
        }

        fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            map_variant!(self, |inner| inner.exiting(event_loop))
        }

        fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            map_variant!(self, |inner| inner.memory_warning(event_loop))
        }
    }
}

fn main() -> Result<()> {
    logging::setup_tracing();
    EventLoop::new()
        .context("creating an event loop")
        .and_then(|event_loop| {
            event_loop
                .run_app(&mut app::AppMode::new(WindowAttributes::default()))
                .context("running an app")
        })
        .context("running an engine")
}
