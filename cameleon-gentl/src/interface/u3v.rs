use cameleon_impl::memory::prelude::*;

use crate::{port::*, GenTlResult};

use super::{u3v_memory as memory, Interface};

const INTERFACE_ID: &str = "Cameleon-U3V-Interface-Module";

pub struct U3VInterfaceModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,
}

impl U3VInterfaceModule {
    pub fn new() -> Self {
        let port_info = PortInfo {
            id: INTERFACE_ID.into(),
            vendor: memory::GenApi::vendor_name().into(),
            tl_type: TlType::USB3Vision,
            module_type: ModuleType::Interface,
            endianness: Endianness::LE,
            access: PortAccess::RW,
            version: memory::GenApi::genapi_version(),
            port_name: "InterfacePort".into(),
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
        }
    }

    fn handle_events(&mut self) {
        todo!()
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
