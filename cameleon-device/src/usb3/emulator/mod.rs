mod channel;
mod device;
mod peripheral;

pub use channel::{ControlChannel, ReceiveChannel};
pub use device::Device;
pub use peripheral::{BuilderError, BuilderResult, EmulatorBuilder};

use crate::usb3::Result;

pub fn enumerate_device() -> Result<Vec<Device>> {
    let device_ids = peripheral::DevicePool::with(|pool| pool.device_ids());
    let mut devices = Vec::with_capacity(device_ids.len());

    for id in device_ids {
        let info = match peripheral::DevicePool::with(|pool| pool.device_info(id)) {
            Ok(info) => info,
            Err(_) => continue,
        };

        devices.push(Device::new(id, info));
    }

    Ok(devices)
}
