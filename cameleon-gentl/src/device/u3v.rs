use std::sync::{Arc, Mutex};

use cameleon_device::u3v as usb3;
use cameleon_impl::memory::prelude::*;

use crate::{port::*, GenTlError, GenTlResult};

use super::{u3v_memory as memory, DeviceAccessStatus};

pub(crate) fn enumerate_u3v_device() -> GenTlResult<Vec<Arc<Mutex<U3VDeviceModule>>>> {
    Ok(usb3::enumerate_devices()?
        .into_iter()
        .map(|dev| Arc::new(U3VDeviceModule::new(dev).into()))
        .collect())
}

pub struct U3VDeviceModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,

    device: usb3::Device,
    ctrl_channel: Option<Arc<Mutex<usb3::ControlChannel>>>,
    event_channel: Option<Arc<Mutex<usb3::ReceiveChannel>>>,
    stream_channel: Option<Arc<Mutex<usb3::ReceiveChannel>>>,

    /// Current status of the device.  
    /// `DeviceAccessStatus` and `DeviceAccessStatusReg` in VM doesn't reflect this value while
    /// [`Interface::UpdateDeviceList`] is called as the GenTL specification describes.
    current_status: memory::GenApi::DeviceAccessStatus,
}

// TODO: Implement methods for stream and event channel.
impl U3VDeviceModule {
    /// Close the remote device.
    /// All channels to the remote device are invalidated.
    ///
    /// In order to open the device, please call [cameleon_gentl::interface::u3v::U3VInterfaceModule::open_device] as
    /// GenTL specification describes.
    pub fn close(&mut self) -> GenTlResult<()> {
        let current_status: DeviceAccessStatus = self.current_status.into();
        if !current_status.is_opened() {
            return Ok(());
        }

        if let Some(ctrl_channel) = self.ctrl_channel.take() {
            ctrl_channel.lock().unwrap().close()?;
        }

        if let Some(event_channel) = self.event_channel.take() {
            event_channel.lock().unwrap().close()?;
        }

        if let Some(stream_channel) = self.stream_channel.take() {
            stream_channel.lock().unwrap().close()?;
        }

        self.current_status = memory::GenApi::DeviceAccessStatus::ReadWrite;

        Ok(())
    }

    /// NOTE: Unlike another module of GenTL, this methods doesn't initialize VM registers due to spec requirements.
    /// Initialization of VM registers is done in [`U3VDeviceModule::open`] method.
    pub(crate) fn new(device: usb3::Device) -> Self {
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
            ctrl_channel: None,
            event_channel: None,
            stream_channel: None,

            current_status: memory::GenApi::DeviceAccessStatus::Unknown,
        }
    }

    /// Try to open the remote device and initialize VM regiteres.
    pub(crate) fn open(&mut self) -> GenTlResult<()> {
        let current_status: DeviceAccessStatus = self.current_status.into();
        if current_status.is_opened() {
            return Err(GenTlError::ResourceInUse);
        }

        macro_rules! try_open {
            ($channel:ident) => {
                if let Err(e) = $channel.open() {
                    match e {
                        usb3::Error::LibUsbError(usb3::LibUsbError::Busy) => {
                            self.current_status = memory::GenApi::DeviceAccessStatus::Busy;
                        }
                        _ => {
                            self.current_status = memory::GenApi::DeviceAccessStatus::NoAccess;
                        }
                    }
                    return Err(e.into());
                }
            };
        }

        let mut ctrl_channel = self.device.control_channel()?;
        try_open!(ctrl_channel);

        let mut event_channel = self.device.event_channel()?;
        if let Some(event_channel) = &mut event_channel {
            try_open!(event_channel);
        }

        let mut stream_channel = self.device.stream_channel()?;
        if let Some(stream_channel) = &mut stream_channel {
            try_open!(stream_channel);
        }

        self.ctrl_channel = Some(Arc::new(ctrl_channel.into()));
        self.event_channel = event_channel.map(|c| Arc::new(c.into()));
        self.stream_channel = stream_channel.map(|c| Arc::new(c.into()));
        self.current_status = memory::GenApi::DeviceAccessStatus::OpenReadWrite;

        Ok(())
    }

    pub(crate) fn device_info(&self) -> &usb3::DeviceInfo {
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

    fn port_info(&self) -> &PortInfo {
        &self.port_info
    }

    fn xml_infos(&self) -> &[XmlInfo] {
        &self.xml_infos
    }
}

impl From<usb3::Error> for GenTlError {
    fn from(err: usb3::Error) -> Self {
        GenTlError::IoError(err.into())
    }
}
