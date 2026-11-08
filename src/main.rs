#![allow(clippy::unit_arg, reason = "I like it")]
#![allow(clippy::option_map_unit_fn, reason = "I like it")]
#![allow(clippy::single_match, reason = "I like it")]
#![recursion_limit = "1024"]

use {
    crate::{app::App, wgpu_ext::init_once::EventLoopInitExt},
    anyhow::{Context, Result},
    tap::{Pipe, TapFallible},
    winit::{dpi::Size, event_loop::EventLoop, window::WindowAttributes},
};

mod app;
mod logging;
mod utils;
mod wgpu_ext;

fn main() -> Result<()> {
    logging::setup_tracing();

    EventLoop::new()
        .context("creating an event loop")
        .and_then(|event_loop| {
            event_loop
                .run_app_init(|event_loop| {
                    App::new(
                        WindowAttributes::default().with_inner_size(Size::Physical(winit::dpi::PhysicalSize { width: 800, height: 600 })),
                        event_loop,
                    )
                    .pipe(futures::executor::block_on)
                    .context("initializing app")
                    .tap_err(|e| tracing::error!("{e:#?}"))
                    .tap_err(|_| event_loop.exit())
                    .ok()
                })
                .context("running an app")
        })
        .context("running an engine")
}
