use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cameleon::device::CompressionType;
use cameleon_impl::memory::{prelude::*, MemoryObserver};

use crate::{
    imp::{
        device::{
            u3v::{enumerate_u3v_device, U3VDeviceModule},
            Device, DeviceAccessStatus,
        },
        port::*,
    },
    GenTlError, GenTlResult,
};

use super::{u3v_memory as memory, Interface};

const INTERFACE_ID: &str = "Cameleon-U3V-Interface-Module";

pub(crate) struct U3VInterfaceModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,
    is_opened: bool,

    devices: Vec<Box<Mutex<U3VDeviceModule>>>,
    event_queue: Arc<Mutex<VecDeque<MemoryEvent>>>,
}

impl U3VInterfaceModule {
    pub(crate) fn new() -> Self {
        let port_info = PortInfo {
            id: INTERFACE_ID.into(),
            vendor: memory::GenApi::vendor_name().into(),
            tl_type: memory::GenApi::InterfaceType::USB3Vision.into(),
            module_type: ModuleType::Interface,
            endianness: Endianness::LE,
            access: PortAccess::RW,
            version: memory::GenApi::genapi_version(),
            port_name: memory::GenApi::InterfacePort.into(),
        };

        let xml_info = XmlInfo {
            location: XmlLocation::RegisterMap {
                address: memory::GenApi::xml_address(),
                size: memory::GenApi::xml_length(),
            },
            schema_version: memory::GenApi::schema_version(),
            compressed: CompressionType::Uncompressed,
        };

        let mut module = Self {
            vm: memory::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],
            is_opened: false,

            devices: vec![],
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        };

        module.initialize_vm();
        module
    }

    fn update_device_list(&mut self) -> GenTlResult<bool> {
        self.assert_open()?;

        // First, reflect current device status.
        for device in &self.devices {
            device.lock().unwrap().reflect_status();
        }

        // Enumerate devices connected to the interface.
        let found_devices = enumerate_u3v_device()?
            .into_iter()
            .map(|dev| Box::new(Mutex::new(dev)));

        let mut changed = false;

        for found_device in found_devices {
            let found_device_guard = found_device.lock().unwrap();
            let id = found_device_guard.device_id();

            if let Some(device) = self.find_device_by_id(id)? {
                // If device has already been found and its current status is NoAccess, then close
                // it and change its status to Unknown(initial state).
                let mut device_guard = device.lock().unwrap();
                if device_guard.access_status() == DeviceAccessStatus::NoAccess {
                    device_guard.close().ok();
                    device_guard.force_access_status(DeviceAccessStatus::Unknown);
                    changed = true;
                }
            } else {
                // If device hasn't been found, then just add it to device pool.
                drop(found_device_guard);
                self.devices.push(found_device);
                changed = true;
            }
        }

        if changed {
            for device in &self.devices {
                let mut device_guard = device.lock().unwrap();
                device_guard.reflect_status();
            }

            self.vm
                .write::<memory::GenApi::DeviceSelectorMaxReg>(self.devices.len() as u32 - 1)
                .unwrap();
            self.handle_device_selector_change()?;
        }

        Ok(changed)
    }

    fn assert_open(&self) -> GenTlResult<()> {
        if self.is_opened {
            Ok(())
        } else {
            Err(GenTlError::NotInitialized)
        }
    }

    fn find_device_by_id(&self, id: &str) -> GenTlResult<Option<&Mutex<U3VDeviceModule>>> {
        for dev in self.devices.iter() {
            if &dev.lock().unwrap().port_info()?.id == id {
                return Ok(Some(dev.as_ref()));
            }
        }

        Ok(None)
    }

    fn initialize_vm(&mut self) {
        use memory::GenApi;

        self.vm
            .write::<GenApi::InterfaceIDReg>(INTERFACE_ID.into())
            .unwrap();
        self.vm.write::<GenApi::DeviceSelectorMaxReg>(0).unwrap();
        self.vm.write::<GenApi::DeviceSelectorReg>(0).unwrap();

        self.register_observers();
    }

    fn register_observers(&mut self) {
        let device_update_observer = DeviceUpdateListRegObserver(self.event_queue.clone());
        self.vm
            .register_observer::<memory::GenApi::DeviceUpdateListReg, _>(device_update_observer);

        let device_selector_observer = DeviceSelectorRegObserver(self.event_queue.clone());
        self.vm
            .register_observer::<memory::GenApi::DeviceSelectorReg, _>(device_selector_observer);
    }

    fn handle_events(&mut self) -> GenTlResult<()> {
        loop {
            // Drop mutex guard in every iteration to avoid deadlock possibility.
            let event = self.event_queue.lock().unwrap().pop_front();

            match event {
                Some(MemoryEvent::DeviceUpdateList) => {
                    self.update_device_list()?;
                }
                Some(MemoryEvent::DeviceSelector) => self.handle_device_selector_change()?,
                None => break,
            }
        }

        Ok(())
    }

    fn handle_device_selector_change(&mut self) -> GenTlResult<()> {
        use memory::GenApi;

        let device_idx = self.vm.read::<GenApi::DeviceSelectorReg>().unwrap() as usize;

        if device_idx >= self.devices.len() {
            return Err(GenTlError::InvalidIndex);
        }

        let device = self.devices[device_idx].lock().unwrap();
        let device_info = device.device_info();

        self.vm
            .write::<GenApi::DeviceIDReg>(device.port_info()?.id.to_owned())?;

        self.vm
            .write::<GenApi::DeviceVendorNameReg>(device_info.vendor_name.to_owned())
            .unwrap();

        self.vm
            .write::<GenApi::DeviceModelNameReg>(device_info.model_name.to_owned())?;

        let status: GenApi::DeviceAccessStatus = device.access_status().into();
        self.vm
            .write::<GenApi::DeviceAccessStatusReg>(status as u32)?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum MemoryEvent {
    DeviceUpdateList,
    DeviceSelector,
}

#[derive(Clone)]
struct DeviceUpdateListRegObserver(Arc<Mutex<VecDeque<MemoryEvent>>>);
impl MemoryObserver for DeviceUpdateListRegObserver {
    fn update(&self) {
        self.0
            .lock()
            .unwrap()
            .push_back(MemoryEvent::DeviceUpdateList)
    }
}

#[derive(Clone)]
struct DeviceSelectorRegObserver(Arc<Mutex<VecDeque<MemoryEvent>>>);
impl MemoryObserver for DeviceSelectorRegObserver {
    fn update(&self) {
        self.0
            .lock()
            .unwrap()
            .push_back(MemoryEvent::DeviceSelector)
    }
}

impl Port for U3VInterfaceModule {
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

        self.handle_events()?;

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

impl Interface for U3VInterfaceModule {
    fn open(&mut self) -> GenTlResult<()> {
        if self.is_opened {
            Err(GenTlError::ResourceInUse)
        } else {
            self.is_opened = true;
            Ok(())
        }
    }

    fn close(&mut self) -> GenTlResult<()> {
        for dev in &self.devices {
            dev.lock().unwrap().close()?;
        }

        self.is_opened = false;
        Ok(())
    }

    fn interface_id(&self) -> &str {
        INTERFACE_ID
    }

    fn display_name(&self) -> &str {
        "U3V Interface Module"
    }

    fn tl_type(&self) -> TlType {
        memory::GenApi::InterfaceType::USB3Vision.into()
    }

    fn mac_addr(&self) -> Option<[u8; 6]> {
        None
    }

    fn ip_addr(&self) -> Option<std::net::Ipv4Addr> {
        None
    }

    fn subnet_mask(&self) -> Option<std::net::Ipv4Addr> {
        None
    }

    fn gateway_addr(&self) -> Option<std::net::Ipv4Addr> {
        None
    }

    fn devices(&self) -> Vec<&Mutex<dyn Device>> {
        let mut dyn_devices: Vec<&Mutex<dyn Device>> = Vec::with_capacity(self.devices.len());
        for dev in &self.devices {
            dyn_devices.push(dev.as_ref());
        }
        dyn_devices
    }

    // NOTE: We ignore timeout for now.
    fn update_device_list(&mut self, _timeout: std::time::Duration) -> GenTlResult<bool> {
        self.assert_open()?;

        self.update_device_list()
    }
}

impl Default for U3VInterfaceModule {
    fn default() -> Self {
        Self::new()
    }
}
