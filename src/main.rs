use anyhow::{Context, Result};
use tracing::info;
use winit::{application::ApplicationHandler, event_loop::EventLoop};

mod logging;

pub struct App;

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("resumed")
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        info!("window event: {event:#?}")
    }
}

fn main() -> Result<()> {
    logging::setup_tracing();
    EventLoop::new()
        .context("creating an event loop")
        .and_then(|event_loop| event_loop.run_app(&mut App).context("running an app"))
        .context("running an engine")
}
