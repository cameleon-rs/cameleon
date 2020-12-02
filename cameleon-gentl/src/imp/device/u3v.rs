use std::sync::{Arc, Mutex};

use cameleon::device::u3v;
use cameleon_impl::memory::prelude::*;

use crate::{imp::port::*, GenTlError, GenTlResult};

use super::{u3v_memory as memory, Device, DeviceAccessStatus};

pub(crate) fn enumerate_u3v_device() -> GenTlResult<Vec<U3VDeviceModule>> {
    Ok(u3v::enumerate_devices()
        .unwrap()
        .into_iter()
        .map(|dev| U3VDeviceModule::new(dev))
        .collect())
}

pub(crate) struct U3VDeviceModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,

    device: u3v::Device,

    /// Current status of the device.  
    /// `DeviceAccessStatus` and `DeviceAccessStatusReg` in VM doesn't reflect this value while
    /// [`Interface::UpdateDeviceList`] is called as the GenTL specification describes.
    current_status: memory::GenApi::DeviceAccessStatus,
}

// TODO: Implement methods for stream and event channel.
impl U3VDeviceModule {
    /// NOTE: Unlike another module of GenTL, this methods doesn't initialize VM registers due to spec requirements.
    /// Initialization of VM registers is done in [`U3VDeviceModule::open`] method.
    pub(crate) fn new(device: u3v::Device) -> Self {
        let device_info = device.device_info();

        let port_info = PortInfo {
            id: device_info.guid.clone(),
            vendor: memory::GenApi::vendor_name().into(),
            tl_type: memory::GenApi::DeviceType::USB3Vision.into(),
            module_type: ModuleType::Device,
            endianness: Endianness::LE,
            access: PortAccess::RW,
            version: memory::GenApi::genapi_version(),
            port_name: memory::GenApi::DevicePort.into(),
        };

        let xml_info = XmlInfo {
            location: XmlLocation::RegisterMap {
                address: memory::GenApi::xml_address(),
                size: memory::GenApi::xml_length(),
            },
            schema_version: memory::GenApi::schema_version(),
            compressed: Compressed::None,
        };

        Self {
            vm: memory::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],

            device,

            current_status: memory::GenApi::DeviceAccessStatus::Unknown,
        }
    }

    pub(crate) fn device_info(&self) -> &u3v::DeviceInfo {
        self.device.device_info()
    }

    /// Reflect current_status to `DeviceAccessStatusReg` in VM.
    /// Actual current status of the device isn't visible until this method is called.
    /// See GenTL specification for more details.
    pub(crate) fn reflect_status(&mut self) {
        self.vm
            .write::<memory::GenApi::DeviceAccessStatusReg>(self.current_status as u32)
            .unwrap();
    }

    /// Access status of the device. Returned status is same value as `DeviceAccessStatusReg`.
    /// Make sure to call [`U3VDeviceModule::reflect_status`] to obtain up to date status before
    /// calling [`U3VDeviceModule::access_status`].  
    /// See GenTL specification for more details.
    pub(crate) fn access_status(&self) -> DeviceAccessStatus {
        let raw_value = self
            .vm
            .read::<memory::GenApi::DeviceAccessStatusReg>()
            .unwrap();
        memory::GenApi::DeviceAccessStatus::from_num(raw_value as isize).into()
    }

    pub(crate) fn device_id(&self) -> &str {
        &self.device_info().guid
    }

    pub(crate) fn force_access_status(&mut self, status: DeviceAccessStatus) {
        let status: memory::GenApi::DeviceAccessStatus = status.into();
        self.current_status = status;
        self.reflect_status();
    }

    fn assert_open(&self) -> GenTlResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(GenTlError::NotInitialized)
        }
    }

    fn is_opened(&self) -> bool {
        let current_status: DeviceAccessStatus = self.current_status.into();
        current_status.is_opened()
    }

    fn handle_events(&mut self) {
        todo!()
    }
}

impl Drop for U3VDeviceModule {
    fn drop(&mut self) {
        self.close().ok();
    }
}

impl Port for U3VDeviceModule {
    fn read(&self, address: u64, size: usize) -> GenTlResult<Vec<u8>> {
        let address = address as usize;
        Ok(self
            .vm
            .read_raw(address..size + address)
            .map(|v| v.to_owned())?)
    }

    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<()> {
        self.vm.write_raw(address as usize, &data)?;

        self.handle_events();

        Ok(())
    }

    fn port_info(&self) -> GenTlResult<&PortInfo> {
        // TODO: open assertion.
        Ok(&self.port_info)
    }

    fn xml_infos(&self) -> GenTlResult<&[XmlInfo]> {
        // TODO: open assertion.
        Ok(&self.xml_infos)
    }
}

impl Device for U3VDeviceModule {
    fn open(&mut self) -> GenTlResult<()> {
        if self.is_opened() {
            return Err(GenTlError::ResourceInUse);
        }

        let res: GenTlResult<()> = self.device.open().map_err(Into::into);

        self.current_status = match &res {
            Ok(()) => memory::GenApi::DeviceAccessStatus::OpenReadWrite,
            Err(GenTlError::AccessDenied) => memory::GenApi::DeviceAccessStatus::Busy,
            Err(GenTlError::Io(..)) => memory::GenApi::DeviceAccessStatus::NoAccess,
            _ => memory::GenApi::DeviceAccessStatus::Unknown,
        };

        res
    }

    fn close(&mut self) -> GenTlResult<()> {
        let res: GenTlResult<()> = self.device.close().map_err(Into::into);
        self.current_status = match res {
            Ok(()) => memory::GenApi::DeviceAccessStatus::ReadWrite,
            Err(GenTlError::Io(..)) => memory::GenApi::DeviceAccessStatus::NoAccess,
            _ => memory::GenApi::DeviceAccessStatus::Unknown,
        };

        Ok(())
    }
}
