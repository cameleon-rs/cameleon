use std::sync::{Arc, Mutex};

use async_std::sync::{Receiver, Sender};
use lazy_static::lazy_static;

use super::{
    device::Device,
    fake_protocol::{FakeAckPacket, FakeReqPacket},
};

lazy_static! {
    pub(crate) static ref DEVICE_CONTEXT: Arc<Mutex<DeviceContext>> =
        Arc::new(Mutex::new(DeviceContext::new()));
}

pub(crate) struct DeviceContext {
    devices: Vec<Device>,
}

impl DeviceContext {
    pub(crate) fn connect(
        &mut self,
        device_idx: usize,
    ) -> Option<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)> {
        self.devices
            .get_mut(device_idx)
            .map(|device| device.connect())
            .flatten()
    }

    pub(crate) fn num_devices(&self) -> usize {
        self.devices.len()
    }

    pub(super) fn attach_device(&mut self, mut device: Device) {
        device.run();
        self.devices.push(device);
    }

    const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
}
