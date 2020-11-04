use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cameleon_impl::memory::{prelude::*, MemoryObserver};

use super::{port::*, GenTlResult};

use crate::interface::{u3v::U3VInterfaceModule, Interface};

mod memory;

const NUM_INTERFACE: usize = 1;

pub struct SystemModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,
    interfaces: [Arc<Mutex<dyn Interface>>; NUM_INTERFACE],
    event_queue: Arc<Mutex<VecDeque<MemoryEvent>>>,
}

impl SystemModule {
    pub fn new() -> Self {
        let port_info = PortInfo {
            id: memory::GenApi::TLID.into(),
            vendor: memory::GenApi::vendor_name().into(),
            tl_type: TlType::Mixed,
            module_type: ModuleType::System,
            endianness: Endianness::LE,
            access: PortAccess::RW,
            version: memory::GenApi::genapi_version(),
            port_name: "TLPort".into(),
        };

        let xml_info = XmlInfo {
            location: XmlLocation::RegisterMap {
                address: memory::GenApi::xml_address(),
                size: memory::GenApi::xml_length(),
            },
            schema_version: memory::GenApi::schema_version(),
            compressed: Compressed::None,
        };

        let mut system_module = Self {
            vm: memory::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],
            interfaces: [Arc::new(Mutex::new(U3VInterfaceModule::new()))],
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        };

        system_module.initialize_vm();
        system_module
    }

    pub fn interfaces(&self) -> &[Arc<Mutex<dyn Interface>>] {
        &self.interfaces
    }

    fn initialize_vm(&mut self) {
        use memory::GenApi;

        self.vm.write::<GenApi::TLPathReg>(file!().into()).unwrap();
        self.vm.write::<GenApi::InterfaceSelectorReg>(0).unwrap();
        self.vm
            .write::<GenApi::InterfaceSelectorMaxReg>(NUM_INTERFACE as u32)
            .unwrap();

        self.register_observers();
    }

    fn register_observers(&mut self) {
        let interface_update_observer = InterfaceUpdateListRegObserver(self.event_queue.clone());
        self.vm
            .register_observer::<memory::GenApi::InterfaceUpdateListReg, _>(
                interface_update_observer,
            );

        let interface_selector_observer = InterfaceSelectorRegObserver(self.event_queue.clone());
        self.vm
            .register_observer::<memory::GenApi::InterfaceSelectorReg, _>(
                interface_selector_observer,
            );
    }

    fn handle_events(&mut self) {
        loop {
            // Drop mutex guard in every iteration to avoid deadlock possibility.
            let event = self.event_queue.lock().unwrap().pop_front();

            match event {
                // We don't implement dynamic update of interfaces.
                Some(MemoryEvent::InterfaceUpdateList) => {}
                Some(MemoryEvent::InterfaceSelector) => self.handle_interface_selector_change(),
                None => break,
            }
        }
    }

    fn handle_interface_selector_change(&mut self) {
        use memory::GenApi;

        let interface_idx = self.vm.read::<GenApi::InterfaceSelectorReg>().unwrap() as usize;

        // Specified interface doesn't exist. In that case, just ignore.
        if interface_idx >= self.interfaces.len() {
            return;
        }

        let interface = &self.interfaces[interface_idx].lock().unwrap();

        let unique_id = interface.unique_id();
        self.vm
            .write::<GenApi::InterfaceIDReg>(unique_id.into())
            .unwrap();

        macro_rules! byte_array_to_int {
            ($array:expr, $array_size:literal, $result_ty: ty) => {{
                let mut result: $result_ty = 0;
                for i in 0..$array_size {
                    result += $array[i] as $result_ty;
                    result <<= 8;
                }
                result
            }};
        }

        if let Some(mac_addr) = interface.mac_addr() {
            let mac_addr = byte_array_to_int!(mac_addr, 6, u64);
            self.vm
                .write::<GenApi::GevInterfaceMACAddressReg>(mac_addr)
                .unwrap();
        }

        if let Some(ip_addr) = interface.ip_addr() {
            let ip_addr = byte_array_to_int!(ip_addr.octets(), 4, u32);
            self.vm
                .write::<GenApi::GevInterfaceDefaultIPAddressReg>(ip_addr)
                .unwrap();
        }

        if let Some(subnet_mask) = interface.subnet_mask() {
            let subnet_mask = byte_array_to_int!(subnet_mask.octets(), 4, u32);
            self.vm
                .write::<GenApi::GevInterfaceDefaultSubnetMaskReg>(subnet_mask)
                .unwrap();
        }

        if let Some(gateway_addr) = interface.gateway_addr() {
            let gateway_addr = byte_array_to_int!(gateway_addr.octets(), 4, u32);
            self.vm
                .write::<GenApi::GevInterfaceDefaultGatewayReg>(gateway_addr)
                .unwrap();
        }
    }
}

impl Default for SystemModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Port for SystemModule {
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

#[derive(Clone, Copy)]
enum MemoryEvent {
    InterfaceUpdateList,
    InterfaceSelector,
}

#[derive(Clone)]
struct InterfaceUpdateListRegObserver(Arc<Mutex<VecDeque<MemoryEvent>>>);
impl MemoryObserver for InterfaceUpdateListRegObserver {
    fn update(&self) {
        self.0
            .lock()
            .unwrap()
            .push_back(MemoryEvent::InterfaceUpdateList)
    }
}

#[derive(Clone)]
struct InterfaceSelectorRegObserver(Arc<Mutex<VecDeque<MemoryEvent>>>);
impl MemoryObserver for InterfaceSelectorRegObserver {
    fn update(&self) {
        self.0
            .lock()
            .unwrap()
            .push_back(MemoryEvent::InterfaceSelector)
    }
}
