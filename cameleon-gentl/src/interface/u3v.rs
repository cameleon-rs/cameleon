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
    fn interface_id(&self) -> &str {
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

impl Default for U3VInterfaceModule {
    fn default() -> Self {
        Self::new()
    }
}
