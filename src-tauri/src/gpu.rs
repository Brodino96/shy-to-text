use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub id: i32,
    pub name: String,
    pub device_type: String,
    pub backend: String,
}

pub fn get_gpu_devices() -> Vec<GpuDevice> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });

    let adapters: Vec<wgpu::Adapter> = instance.enumerate_adapters(wgpu::Backends::VULKAN);

    adapters
        .into_iter()
        .enumerate()
        .map(|(index, adapter)| {
            let info = adapter.get_info();
            GpuDevice {
                id: index as i32,
                name: info.name,
                device_type: format!("{:?}", info.device_type),
                backend: format!("{:?}", info.backend),
            }
        })
        .collect()
}
