/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryFrom, sync::Mutex};

use cameleon::{
    genapi::{CompressionType, SharedDefaultGenApiCtxt},
    u3v::{self, SharedControlHandle, StreamHandle},
};
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

type Camera = cameleon::Camera<SharedControlHandle, StreamHandle, SharedDefaultGenApiCtxt>;

pub(crate) fn enumerate_u3v_device() -> GenTlResult<Vec<U3VDeviceModule>> {
    todo!()
}

pub(crate) struct U3VDeviceModule {
    vm: genapi::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,

    camera: Camera,
    remote_device: Option<Box<Mutex<U3VRemoteDevice>>>,

    /// Current status of the device.  
    /// `DeviceAccessStatus` and `DeviceAccessStatusReg` in VM doesn't reflect this value while
    /// [`Interface::UpdateDeviceList`] is called as the GenTL specification describes.
    current_status: super::DeviceAccessStatus,
}

// TODO: Implement methods for stream and event channel.
impl U3VDeviceModule {
    pub(crate) fn new(camera: Camera) -> GenTlResult<Self> {
        let device_info = camera.ctrl.device_info();

        let port_info = PortInfo {
            id: device_info.guid,
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

            camera,
            remote_device: None,

            current_status: super::DeviceAccessStatus::Unknown,
        };

        dev.initialize_vm()?;
        Ok(dev)
    }

    pub(crate) fn device_info(&self) -> &u3v::DeviceInfo {
        todo!()
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
        todo!()
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

        self.vm.write_raw(address as usize, data)?;
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
        todo!()
    }

    fn close(&mut self) -> GenTlResult<()> {
        todo!()
    }

    fn device_id(&self) -> &str {
        &self.port_info.id
    }

    fn remote_device(&self) -> GenTlResult<&Mutex<dyn Port>> {
        self.assert_open()?;

        Ok(self.remote_device.as_ref().unwrap().as_ref())
    }

    fn vendor_name(&self) -> GenTlResult<String> {
        todo!()
    }

    fn model_name(&self) -> GenTlResult<String> {
        todo!()
    }

    fn display_name(&self) -> GenTlResult<String> {
        let vendor = self.vendor_name()?;
        let model_name = self.model_name()?;
        let id = self.device_id();
        Ok(format!("{vendor} {model_name} ({id})"))
    }

    fn tl_type(&self) -> TlType {
        TlType::USB3Vision
    }

    fn device_access_status(&self) -> DeviceAccessStatus {
        todo!()
    }

    fn user_defined_name(&self) -> GenTlResult<String> {
        todo!()
    }

    fn serial_number(&self) -> GenTlResult<String> {
        Ok(self.device_info().serial_number.clone())
    }

    fn device_version(&self) -> GenTlResult<String> {
        todo!()
    }

    fn timespamp_frequency(&self) -> GenTlResult<u64> {
        todo!()
    }
}

pub(crate) struct U3VRemoteDevice {}

impl U3VRemoteDevice {
    fn new(handle: u3v::ControlHandle) -> GenTlResult<Self> {
        todo!()
    }

    fn port_info(handle: &u3v::ControlHandle) -> GenTlResult<PortInfo> {
        todo!()
    }

    fn xml_infos(handle: &u3v::ControlHandle) -> GenTlResult<Vec<XmlInfo>> {
        todo!()
    }
}

impl Port for U3VRemoteDevice {
    fn read(&self, address: u64, buf: &mut [u8]) -> GenTlResult<usize> {
        todo!()
    }

    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<usize> {
        todo!()
    }

    fn port_info(&self) -> GenTlResult<&PortInfo> {
        todo!()
    }

    fn xml_infos(&self) -> GenTlResult<&[XmlInfo]> {
        todo!()
    }
}
