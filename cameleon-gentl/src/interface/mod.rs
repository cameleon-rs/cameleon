use crate::port::Port;

pub mod u3v;

mod u3v_memory;

// TODO: Add device related functions.
pub trait Interface: Port {
    fn interface_id(&self) -> &str;

    fn mac_addr(&self) -> Option<[u8; 6]>;

    fn ip_addr(&self) -> Option<std::net::Ipv4Addr>;

    fn subnet_mask(&self) -> Option<std::net::Ipv4Addr>;

    fn gateway_addr(&self) -> Option<std::net::Ipv4Addr>;
}
