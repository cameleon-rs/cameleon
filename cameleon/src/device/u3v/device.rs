use cameleon_device::u3v;

use super::control_handle::ControlHandle;

use crate::device::{DeviceError, DeviceResult};

pub type DeviceInfo = u3v::DeviceInfo;

pub struct Device {
    device: u3v::Device,

    ctrl_handle: ControlHandle,
    // TODO: Stream and event handles.
}

impl Device {
    pub fn new(device: u3v::Device) -> DeviceResult<Self> {
        let ctrl_handle = ControlHandle::new(&device)?;

        Ok(Self {
            device,
            ctrl_handle,
        })
    }

    pub fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.ctrl_handle.open()?;

        Ok(())
    }

    pub fn control_handle(&mut self) -> DeviceResult<ControlHandle> {
        if self.is_opened() {
            Ok(self.ctrl_handle.clone())
        } else {
            Err(DeviceError::NotOpened)
        }
    }

    pub fn is_opened(&self) -> bool {
        self.ctrl_handle.is_opened()
    }

    pub fn close(&mut self) -> DeviceResult<()> {
        self.ctrl_handle.close()
    }

    /// Basic information of the device. No need to call [`Device::open`] to obtain the
    /// information.
    pub fn device_info(&self) -> &DeviceInfo {
        self.device.device_info()
    }
}

/// Enumerate all devices connected to the host.
pub fn enumerate_devices() -> DeviceResult<Vec<Device>> {
    let devices = u3v::enumerate_devices()?;

    Ok(devices
        .into_iter()
        .map(|dev| Device::new(dev))
        .filter_map(|d| d.ok())
        .collect())
}
