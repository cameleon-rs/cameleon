use cameleon_device::u3v;

use crate::device::{DeviceError, DeviceResult};

use super::{
    control_handle::ControlHandle,
    register_map::{Abrm, ManifestTable, Sbrm},
};

pub type DeviceInfo = u3v::DeviceInfo;

pub struct Device {
    device: u3v::Device,

    // TODO: Stream and event handles.
    ctrl_handle: ControlHandle,
}

impl Device {
    pub fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.ctrl_handle.open()?;
        Ok(())
    }

    pub fn close(&mut self) -> DeviceResult<()> {
        self.ctrl_handle.close()
    }

    pub fn control_handle(&mut self) -> DeviceResult<ControlHandle> {
        self.assert_open()?;

        Ok(self.ctrl_handle.clone())
    }

    pub fn is_opened(&self) -> bool {
        self.ctrl_handle.is_opened()
    }

    /// Get Technology Agnostic Boot Register Map of the device.
    pub fn abrm(&self) -> DeviceResult<Abrm> {
        self.assert_open()?;

        Abrm::new(&self.ctrl_handle)
    }

    /// Basic information of the device. No need to call [`Device::open`] to obtain the
    /// information.
    pub fn device_info(&self) -> &DeviceInfo {
        self.device.device_info()
    }

    fn new(device: u3v::Device) -> DeviceResult<Self> {
        let ctrl_handle = ControlHandle::new(&device)?;

        Ok(Self {
            device,
            ctrl_handle,
        })
    }

    fn assert_open(&self) -> DeviceResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(DeviceError::NotOpened)
        }
    }
}

/// Enumerate all devices connected to the host.
pub fn enumerate_devices() -> DeviceResult<Vec<Device>> {
    let devices = u3v::enumerate_devices()?;

    Ok(devices
        .into_iter()
        .map(Device::new)
        .filter_map(|d| d.ok())
        .collect())
}

impl Drop for Device {
    fn drop(&mut self) {
        // TODO: log.
        let _ = self.close();
    }
}
