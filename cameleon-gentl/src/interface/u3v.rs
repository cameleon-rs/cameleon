use super::{u3v_memory, Interface};

pub struct U3VInterfaceModule {
    _vm: u3v_memory::Memory,
}

impl U3VInterfaceModule {
    pub fn new() -> Self {
        Self {
            _vm: u3v_memory::Memory::new(),
        }
    }
}

impl Interface for U3VInterfaceModule {
    fn unique_id(&self) -> &'static str {
        "Cameleon-U3V-Interface-Module"
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
