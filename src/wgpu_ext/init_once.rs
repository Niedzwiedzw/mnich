use {
    std::result::Result,
    winit::{
        application::ApplicationHandler,
        error::EventLoopError,
        event_loop::{ActiveEventLoop, EventLoop},
    },
};

macro_rules! forward_running {
    ($self:expr, | $inner:ident | $map:expr) => {
        match $self {
            InitOnceApp::Initializing(_) => {
                tracing::debug!(
                    "init-once runner is not initialized, this would have been dropped: {} ({}:{})",
                    stringify!($map),
                    file!(),
                    line!()
                );
            }
            InitOnceApp::Running($inner) => $map,
        }
    };
}

#[derive(Debug)]
pub enum InitOnceApp<Init, Inner> {
    Initializing(Option<Init>),
    Running(Inner),
}

impl<Init, Inner> InitOnceApp<Init, Inner> {
    pub fn new(init: Init) -> Self {
        InitOnceApp::Initializing(Some(init))
    }
}

impl<T, Init, Inner> ApplicationHandler<T> for InitOnceApp<Init, Inner>
where
    T: 'static,
    Inner: ApplicationHandler<T>,
    Init: FnOnce(&ActiveEventLoop) -> Option<Inner> + 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            InitOnceApp::Initializing(init) => {
                if let Some(inner) = (init.take().expect("initialization only happens once"))(event_loop) {
                    *self = InitOnceApp::Running(inner);
                }
            }
            InitOnceApp::Running(running) => running.resumed(event_loop),
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: winit::event::WindowEvent) {
        forward_running!(self, |inner| inner.window_event(event_loop, window_id, event));
    }

    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
        forward_running!(self, |inner| inner.new_events(event_loop, cause));
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: T) {
        forward_running!(self, |inner| inner.user_event(event_loop, event));
    }

    fn device_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        forward_running!(self, |inner| inner.device_event(event_loop, device_id, event));
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        forward_running!(self, |inner| inner.about_to_wait(event_loop))
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        forward_running!(self, |inner| inner.suspended(event_loop))
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        forward_running!(self, |inner| inner.exiting(event_loop))
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        forward_running!(self, |inner| inner.memory_warning(event_loop))
    }
}

pub trait EventLoopInitExt<T>: Sized {
    fn run_app_init<Init, Inner>(self, init: Init) -> Result<(), EventLoopError>
    where
        T: 'static,
        Init: FnOnce(&ActiveEventLoop) -> Option<Inner> + 'static,
        Inner: ApplicationHandler<T>;
}

impl<T> EventLoopInitExt<T> for EventLoop<T> {
    fn run_app_init<Init, Inner>(self, init: Init) -> Result<(), EventLoopError>
    where
        T: 'static,
        Init: FnOnce(&ActiveEventLoop) -> Option<Inner> + 'static,
        Inner: ApplicationHandler<T>,
    {
        self.run_app(&mut InitOnceApp::new(init))
    }
}
