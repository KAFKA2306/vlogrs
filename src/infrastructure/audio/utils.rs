use cpal::traits::{DeviceTrait, HostTrait};
use tracing::info;
pub fn list_devices() {
    let host: cpal::Host = cpal::default_host();
    let devices = host.input_devices().expect("Failed to list input devices");
    info!("=== Available Audio Input Devices ===");
    for (i, device) in devices.enumerate() {
        let name: String = device.name().unwrap();
        info!("Device {}: {}", i, name);
        if let Ok(mut configs) = device.supported_input_configs() {
            for config in configs.by_ref() {
                info!(
                    "  - Supported: channels={}, min_sample_rate={}, max_sample_rate={}, buffer_size={:?}, sample_format={:?}",
                    config.channels(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    config.buffer_size(),
                    config.sample_format()
                );
            }
        }
    }
}
