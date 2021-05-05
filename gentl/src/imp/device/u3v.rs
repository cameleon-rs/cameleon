use std::{convert::TryFrom, sync::Mutex};

use cameleon::u3v;
use cameleon::CompressionType;
use cameleon_impl::memory::prelude::*;

use crate::{
    imp::{
        genapi_common,
        port::{Endianness, ModuleType, Port, PortAccess, PortInfo, TlType, XmlInfo, XmlLocation},
    },
    GenTlError, GenTlResult,
};

use super::{u3v_genapi as genapi, Device, DeviceAccessStatus};
use genapi::GenApiReg;

pub(crate) fn enumerate_u3v_device() -> GenTlResult<Vec<U3VDeviceModule>> {
    Ok(u3v::enumerate_devices()?
        .into_iter()
        .map(U3VDeviceModule::new)
        .filter_map(std::result::Result::ok)
        .collect())
}

pub(crate) struct U3VDeviceModule {
    vm: genapi::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,

    device: u3v::Device,
    remote_device: Option<Box<Mutex<U3VRemoteDevice>>>,

    /// Current status of the device.  
    /// `DeviceAccessStatus` and `DeviceAccessStatusReg` in VM doesn't reflect this value while
    /// [`Interface::UpdateDeviceList`] is called as the GenTL specification describes.
    current_status: super::DeviceAccessStatus,
}

// TODO: Implement methods for stream and event channel.
impl U3VDeviceModule {
    pub(crate) fn new(device: u3v::Device) -> GenTlResult<Self> {
        let device_info = device.device_info();

        let port_info = PortInfo {
            id: device_info.guid.clone(),
            vendor: genapi::VENDOR_NAME.into(),
            model: genapi::MODEL_NAME.into(),
            tl_type: genapi::DEVICE_TYPE,
            module_type: ModuleType::Interface,
            endianness: Endianness::LE,
            access: PortAccess::RW,
            version: semver::Version::new(
                genapi::XML_MAJOR_VERSION,
                genapi::XML_MINOR_VERSION,
                genapi::XML_SUBMINOR_VERSION,
            ),
            port_name: genapi::PORT_NAME.into(),
        };

        let xml_info = XmlInfo {
            location: XmlLocation::RegisterMap {
                address: genapi::GENAPI_XML_ADDRESS as u64,
                size: genapi::GENAPI_XML_LENGTH,
            },
            schema_version: semver::Version::new(
                genapi_common::SCHEME_MAJOR_VERSION,
                genapi_common::SCHEME_MINOR_VERSION,
                genapi_common::SCHEME_SUBMINOR_VERSION,
            ),
            file_version: semver::Version::new(
                genapi_common::SCHEME_MAJOR_VERSION,
                genapi_common::SCHEME_MINOR_VERSION,
                genapi_common::SCHEME_SUBMINOR_VERSION,
            ),
            sha1_hash: None,
            compressed: CompressionType::Uncompressed,
        };

        let mut dev = Self {
            vm: genapi::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],

            device,
            remote_device: None,

            current_status: super::DeviceAccessStatus::Unknown,
        };

        dev.initialize_vm()?;
        Ok(dev)
    }

    pub(crate) fn device_info(&self) -> &u3v::DeviceInfo {
        self.device.device_info()
    }

    /// Reflect current_status to `DeviceAccessStatusReg` in VM.
    /// Actual current status of the device isn't visible until this method is called.
    /// See GenTL specification for more details.
    pub(crate) fn reflect_status(&mut self) {
        self.vm
            .write::<GenApiReg::DeviceAccessStatus>(self.current_status as u32)
            .unwrap();
    }

    /// Access status of the device. Returned status is same value as `DeviceAccessStatusReg`.
    /// Make sure to call [`U3VDeviceModule::reflect_status`] to obtain up to date status before
    /// calling [`U3VDeviceModule::access_status`].  
    /// See GenTL specification for more details.
    pub(crate) fn access_status(&self) -> DeviceAccessStatus {
        let raw_value = self.vm.read::<GenApiReg::DeviceAccessStatus>().unwrap() as i32;
        // Ok to unwrap because DeviceAccessStatus is RO register.
        DeviceAccessStatus::try_from(raw_value).unwrap()
    }

    pub(crate) fn device_id(&self) -> &str {
        &self.device_info().guid
    }

    pub(crate) fn force_access_status(&mut self, status: DeviceAccessStatus) {
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
        let current_status: DeviceAccessStatus = self.current_status;
        current_status.is_opened()
    }

    #[allow(clippy::unused_self)]
    fn handle_events(&mut self) {
        // TODO: Handle stream related events.
    }

    fn initialize_vm(&mut self) -> GenTlResult<()> {
        self.vm
            .write::<GenApiReg::DeviceID>(self.device_id().into())?;

        let info = self.device.device_info();
        self.vm
            .write::<GenApiReg::DeviceVendorName>(info.vendor_name.clone())?;
        self.vm
            .write::<GenApiReg::DeviceModelName>(info.model_name.clone())?;
        self.reflect_status();

        // TODO: Initialize registeres related to stream.

        Ok(())
    }
}

impl Drop for U3VDeviceModule {
    fn drop(&mut self) {
        self.close().ok();
    }
}

impl Port for U3VDeviceModule {
    fn read(&self, address: u64, buf: &mut [u8]) -> GenTlResult<usize> {
        self.assert_open()?;

        let address = address as usize;
        let len = buf.len();

        let data = self.vm.read_raw(address..address + len)?;
        buf.copy_from_slice(data);

        Ok(len)
    }

    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<usize> {
        self.assert_open()?;

        self.vm.write_raw(address as usize, &data)?;
        self.handle_events();

        Ok(data.len())
    }

    fn port_info(&self) -> GenTlResult<&PortInfo> {
        self.assert_open()?;

        Ok(&self.port_info)
    }

    fn xml_infos(&self) -> GenTlResult<&[XmlInfo]> {
        self.assert_open()?;

        Ok(&self.xml_infos)
    }
}

impl Device for U3VDeviceModule {
    fn open(&mut self, access_flag: super::DeviceAccessFlag) -> GenTlResult<()> {
        if access_flag != super::DeviceAccessFlag::Exclusive {
            return Err(GenTlError::AccessDenied);
        }

        if self.is_opened() {
            return Err(GenTlError::ResourceInUse);
        }

        let ctrl_handle = self.device.control_handle().clone();
        let res: GenTlResult<()> = ctrl_handle.open().map_err(Into::into);
        self.remote_device = Some(Mutex::new(U3VRemoteDevice::new(ctrl_handle)?).into());

        self.current_status = match &res {
            Ok(()) => super::DeviceAccessStatus::OpenReadWrite,
            Err(GenTlError::AccessDenied) => super::DeviceAccessStatus::Busy,
            Err(GenTlError::Io(..)) => super::DeviceAccessStatus::NoAccess,
            _ => super::DeviceAccessStatus::Unknown,
        };

        res
    }

    fn close(&mut self) -> GenTlResult<()> {
        let res: GenTlResult<()> = self.device.control_handle().close().map_err(Into::into);
        self.current_status = match res {
            Ok(()) => super::DeviceAccessStatus::ReadWrite,
            Err(GenTlError::Io(..)) => super::DeviceAccessStatus::NoAccess,
            _ => super::DeviceAccessStatus::Unknown,
        };

        self.remote_device = None;
        Ok(())
    }

    fn device_id(&self) -> &str {
        &self.port_info.id
    }

    fn remote_device(&self) -> GenTlResult<&Mutex<dyn Port>> {
        self.assert_open()?;

        Ok(self.remote_device.as_ref().unwrap().as_ref())
    }

    fn vendor_name(&self) -> GenTlResult<String> {
        Ok(self.device.device_info().vendor_name.clone())
    }

    fn model_name(&self) -> GenTlResult<String> {
        Ok(self.device.device_info().model_name.clone())
    }

    fn display_name(&self) -> GenTlResult<String> {
        let vendor = self.vendor_name()?;
        let model_name = self.model_name()?;
        let id = self.device_id();
        Ok(format!("{} {} ({})", vendor, model_name, id))
    }

    fn tl_type(&self) -> TlType {
        TlType::USB3Vision
    }

    fn device_access_status(&self) -> DeviceAccessStatus {
        self.access_status()
    }

    fn user_defined_name(&self) -> GenTlResult<String> {
        let abrm = self.device.control_handle().abrm()?;
        let user_defined_name = abrm.user_defined_name()?;
        user_defined_name.ok_or(GenTlError::NotAvailable)
    }

    fn serial_number(&self) -> GenTlResult<String> {
        Ok(self.device.device_info().serial_number.clone())
    }

    fn device_version(&self) -> GenTlResult<String> {
        let abrm = self.device.control_handle().abrm()?;
        Ok(abrm.device_version()?)
    }

    fn timespamp_frequency(&self) -> GenTlResult<u64> {
        let abrm = self.device.control_handle().abrm()?;

        //  U3V's Timestamp increment represents ns / tick.
        let timestamp_increment = abrm.timestamp_increment()?;
        Ok((1_000_000_000) / timestamp_increment)
    }
}

pub(crate) struct U3VRemoteDevice {
    handle: u3v::ControlHandle,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,
}

impl U3VRemoteDevice {
    fn new(handle: u3v::ControlHandle) -> GenTlResult<Self> {
        let port_info = Self::port_info(&handle)?;
        let xml_infos = Self::xml_infos(&handle)?;
        Ok(Self {
            handle,
            port_info,
            xml_infos,
        })
    }

    fn port_info(handle: &u3v::ControlHandle) -> GenTlResult<PortInfo> {
        let abrm = handle.abrm()?;

        let id = abrm.serial_number()?;
        let vendor = abrm.manufacturer_name()?;
        let model = abrm.model_name()?;
        let tl_type = TlType::USB3Vision;
        let module_type = ModuleType::RemoteDevice;
        let endianness = Endianness::LE;
        let access = PortAccess::RW;
        let version = abrm.gencp_version()?;
        let port_name = "Device".into();

        Ok(PortInfo {
            id,
            vendor,
            model,
            tl_type,
            module_type,
            endianness,
            access,
            version,
            port_name,
        })
    }

    fn xml_infos(handle: &u3v::ControlHandle) -> GenTlResult<Vec<XmlInfo>> {
        let abrm = handle.abrm()?;
        let manifest_table = abrm.manifest_table()?;

        let mut xml_infos = vec![];
        for ent in manifest_table.entries()? {
            let file_address = ent.file_address()?;
            let file_size = ent.file_size()?;
            let file_info = ent.file_info()?;

            let schema_version = file_info.schema_version();
            let compressed = file_info.compression_type()?;

            let sha1_hash = ent.sha1_hash()?;
            let file_version = ent.genicam_file_version()?;

            let info = XmlInfo {
                location: XmlLocation::RegisterMap {
                    address: file_address,
                    size: file_size as usize,
                },
                schema_version,
                file_version,
                sha1_hash,
                compressed,
            };
            xml_infos.push(info);
        }

        Ok(xml_infos)
    }
}

impl Port for U3VRemoteDevice {
    fn read(&self, address: u64, buf: &mut [u8]) -> GenTlResult<usize> {
        self.handle.read_mem(address, buf)?;
        Ok(buf.len())
    }

    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<usize> {
        self.handle.write_mem(address, data)?;
        Ok(data.len())
    }

    fn port_info(&self) -> GenTlResult<&PortInfo> {
        Ok(&self.port_info)
    }

    fn xml_infos(&self) -> GenTlResult<&[XmlInfo]> {
        Ok(&self.xml_infos)
    }
}
