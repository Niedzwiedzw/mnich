use {
    crate::line_label,
    anyhow::{Context, Result},
    futures::{FutureExt, TryFutureExt},
    next_tuple::NextTuple,
    std::{future::ready, sync::OnceLock},
    tap::Pipe,
    wgpu::{Adapter, Backends, Device, Instance, InstanceFlags, Queue},
};

#[derive(Debug)]
pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl Gpu {
    pub async fn new() -> Result<Self> {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        })
        .pipe(async |instance| {
            instance
                .enumerate_adapters(Backends::all())
                .map(Ok)
                .and_then(|adapters| {
                    adapters
                        .into_iter()
                        .next()
                        .context("no adapters")
                        .pipe(ready)
                })
                .and_then(async |adapter| {
                    adapter
                        .request_device(&wgpu::wgt::DeviceDescriptor {
                            label: line_label!(),
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::default(),
                            experimental_features: wgpu::ExperimentalFeatures::disabled(),
                            memory_hints: wgpu::MemoryHints::default(),
                            trace: wgpu::Trace::Off,
                        })
                        .await
                        .context("requesting a device")
                        .map(|p| p.next_tuple(adapter))
                })
                .await
                .map(|p| p.next_tuple(instance))
        })
        .await
        .map(|(device, queue, adapter, instance)| Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

static GPU: OnceLock<Gpu> = OnceLock::new();

impl Gpu {
    pub fn init_global(self) -> &'static Self {
        if GPU.set(self).is_err() {
            panic!("GPU context already initialized");
        }
        Self::global()
    }

    pub fn global() -> &'static Self {
        GPU.get()
            .expect("GPU context not initialized, run `Gpu::init_global()`")
    }
}
