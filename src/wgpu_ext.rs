pub mod global_context;
pub mod init_once;
pub mod surface;
pub mod command_encoder {
    use {
        crate::wgpu_ext::global_context::Gpu,
        anyhow::{Context, Result},
        std::iter::once,
        tap::Pipe,
        wgpu::CommandEncoder,
    };

    impl Gpu {
        pub fn with_command_encoder(&self, label: &str, with_command_encoder: impl FnOnce(&mut CommandEncoder) -> Result<()>) -> Result<()> {
            let Gpu {
                instance: _,
                adapter: _,
                device,
                queue,
            } = self;

            device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
                .pipe(|mut encoder| {
                    with_command_encoder(&mut encoder)
                        .with_context(|| format!("running operation [{label}] with encoder"))
                        .map(|_| {
                            queue.submit(once(encoder.finish()));
                        })
                })
                .with_context(|| format!("running on encoder: {label}"))
        }

        pub async fn with_command_encoder_async<'task, F>(&self, label: &str, with_command_encoder: F) -> Result<()>
        where
            F: AsyncFnOnce(&mut CommandEncoder) -> Result<()> + 'task,
        {
            let Gpu {
                instance: _,
                adapter: _,
                device,
                queue,
            } = self;

            device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
                .pipe(|mut encoder| async move {
                    with_command_encoder(&mut encoder)
                        .await
                        .with_context(|| format!("running operation [{label}] with encoder"))
                        .map(|_| {
                            queue.submit(once(encoder.finish()));
                        })
                })
                .await
                .with_context(|| format!("running on encoder: {label}"))
        }
    }
}

#[macro_export]
macro_rules! line_label {
    () => {
        Some(concat!(env!("CARGO_PKG_NAME"), " ", file!(), ":", line!()))
    };
}
