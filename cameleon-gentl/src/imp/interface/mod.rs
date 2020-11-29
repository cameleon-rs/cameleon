use crate::{imp::port::Port, GenTlResult};

pub(crate) mod u3v;

mod u3v_memory;

// TODO: Add device related functions.
pub(crate) trait Interface: Port {
    fn open(&mut self) -> GenTlResult<()>;

    fn interface_id(&self) -> &str;

    fn display_name(&self) -> &str;

    fn mac_addr(&self) -> Option<[u8; 6]>;

    fn ip_addr(&self) -> Option<std::net::Ipv4Addr>;

    fn subnet_mask(&self) -> Option<std::net::Ipv4Addr>;

    fn gateway_addr(&self) -> Option<std::net::Ipv4Addr>;
}
