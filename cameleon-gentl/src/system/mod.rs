use cameleon_impl::memory::prelude::*;

use super::{port::*, GenTlResult};

mod memory;

pub struct SystemModule {
    vm: memory::Memory,
    port_info: PortInfo,
    xml_infos: Vec<XmlInfo>,
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

        Self {
            vm: memory::Memory::new(),
            port_info,
            xml_infos: vec![xml_info],
        }
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
        Ok(self
            .vm
            .write_raw(address as usize, &data)
            .map(|v| v.to_owned())?)
    }

    /// Get detailed port information.
    fn port_info(&self) -> &PortInfo {
        &self.port_info
    }

    /// Get available xml infos of the port.
    fn xml_infos(&self) -> &[XmlInfo] {
        &self.xml_infos
    }
}
