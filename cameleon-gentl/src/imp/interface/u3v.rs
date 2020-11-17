use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cameleon_impl::memory::{prelude::*, MemoryObserver};

use crate::imp::{
    device::{
        u3v::{enumerate_u3v_device, U3VDeviceModule},
        DeviceAccessStatus,
    },
    port::*,
    GenTlError, GenTlResult,
};

use super::{u3v_memory as memory, Interface};

const INTERFACE_ID: &str = "Cameleon-U3V-Interface-Module";

pub(crate) struct U3VInterfaceModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,

    devices: Vec<Arc<Mutex<U3VDeviceModule>>>,
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
            compressed: Compressed::None,
        };

        let mut module = Self {
            vm: memory::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],

            devices: vec![],
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        };

        module.initialize_vm();
        module
    }

    pub(crate) fn update_device_list(&mut self) -> GenTlResult<bool> {
        // First, reflect current device status.
        for device in &self.devices {
            device.lock().unwrap().reflect_status();
        }

        // Enumerate devices connected to the interface.
        let found_devices = enumerate_u3v_device()?;

        let mut changed = false;

        for found_device in found_devices {
            let found_device_guard = found_device.lock().unwrap();
            let id = found_device_guard.device_id();

            if let Some(device) = self.find_device_by_id(id) {
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
            self.handle_device_selector_change();
        }

        Ok(changed)
    }

    pub(crate) fn open_device(&self, device_id: &str) -> GenTlResult<Arc<Mutex<U3VDeviceModule>>> {
        let device = self
            .find_device_by_id(device_id)
            .ok_or_else(|| GenTlError::InvalidId(device_id.into()))?;

        self.open_device_impl(device)
    }

    pub(crate) fn open_device_by_idx(
        &self,
        idx: usize,
    ) -> GenTlResult<Arc<Mutex<U3VDeviceModule>>> {
        if idx >= self.devices.len() {
            return Err(GenTlError::InvalidIndex);
        };

        let device = self.devices[idx].clone();
        self.open_device_impl(device)
    }

    pub(crate) fn num_device(&self) -> usize {
        self.devices.len()
    }

    fn open_device_impl(
        &self,
        device: Arc<Mutex<U3VDeviceModule>>,
    ) -> GenTlResult<Arc<Mutex<U3VDeviceModule>>> {
        device.lock().unwrap().open()?;
        Ok(device)
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

    fn handle_events(&mut self) {
        loop {
            // Drop mutex guard in every iteration to avoid deadlock possibility.
            let event = self.event_queue.lock().unwrap().pop_front();

            match event {
                // We don't implement dynamic update of interfaces.
                Some(MemoryEvent::DeviceUpdateList) => {
                    // TODO: Logging.
                    self.update_device_list().ok();
                }
                Some(MemoryEvent::DeviceSelector) => self.handle_device_selector_change(),
                None => break,
            }
        }
    }

    fn find_device_by_id(&self, id: &str) -> Option<Arc<Mutex<U3VDeviceModule>>> {
        self.devices
            .iter()
            .find(|dev| dev.lock().unwrap().port_info().id == id)
            .cloned()
    }

    fn handle_device_selector_change(&mut self) {
        use memory::GenApi;

        let device_idx = self.vm.read::<GenApi::DeviceSelectorReg>().unwrap() as usize;

        if device_idx >= self.devices.len() {
            return;
        }

        let device = self.devices[device_idx].lock().unwrap();
        let device_info = device.device_info();

        self.vm
            .write::<GenApi::DeviceIDReg>(device.port_info().id.to_owned())
            .unwrap();

        self.vm
            .write::<GenApi::DeviceVendorNameReg>(device_info.vendor_name.to_owned())
            .unwrap();

        self.vm
            .write::<GenApi::DeviceModelNameReg>(device_info.model_name.to_owned())
            .unwrap();

        let status: GenApi::DeviceAccessStatus = device.access_status().into();
        self.vm
            .write::<GenApi::DeviceAccessStatusReg>(status as u32)
            .unwrap();
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

impl Interface for U3VInterfaceModule {
    fn interface_id(&self) -> &str {
        INTERFACE_ID
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
}

impl Default for U3VInterfaceModule {
    fn default() -> Self {
        Self::new()
    }
}
