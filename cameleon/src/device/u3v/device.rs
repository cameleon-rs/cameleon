use cameleon_device::u3v;

use crate::device::{DeviceError, DeviceResult};

use super::{
    control_handle::ControlHandle,
    register_map::{Abrm, AbrmStaticData, Sbrm, SbrmStaticData},
};

pub type DeviceInfo = u3v::DeviceInfo;

pub struct Device {
    device: u3v::Device,

    // TODO: Stream and event handles.
    ctrl_handle: ControlHandle,

    abrm: Option<AbrmStaticData>,
    sbrm: Option<SbrmStaticData>,
}

impl Device {
    pub fn new(device: u3v::Device) -> DeviceResult<Self> {
        let ctrl_handle = ControlHandle::new(&device)?;

        Ok(Self {
            device,
            ctrl_handle,
            abrm: None,
            sbrm: None,
        })
    }

    pub fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.ctrl_handle.open()?;
        let abrm = AbrmStaticData::new(&self.ctrl_handle)?;
        let sbrm = SbrmStaticData::new(abrm.sbrm_address, &self.ctrl_handle)?;

        // Initialize transaction configuration using boot register map.
        self.ctrl_handle.initialize_config(&abrm, &sbrm);

        self.abrm = Some(abrm);
        self.sbrm = Some(sbrm);

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

    pub fn abrm(&self) -> DeviceResult<Abrm> {
        if self.is_opened() {
            Ok(Abrm::new(self.abrm.as_ref().unwrap(), &self.ctrl_handle))
        } else {
            Err(DeviceError::NotOpened)
        }
    }

    pub fn sbrm(&self) -> DeviceResult<Sbrm> {
        if self.is_opened() {
            Ok(Sbrm::new(self.sbrm.as_ref().unwrap()))
        } else {
            Err(DeviceError::NotOpened)
        }
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
        .map(Device::new)
        .filter_map(|d| d.ok())
        .collect())
}
