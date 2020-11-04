pub mod u3v;

mod u3v_memory;

// TODO: Add device related functions.
pub trait Interface {
    fn unique_id(&self) -> &'static str;

    fn mac_addr(&self) -> Option<[u8; 6]>;

    fn ip_addr(&self) -> Option<std::net::Ipv4Addr>;

    fn subnet_mask(&self) -> Option<std::net::Ipv4Addr>;

    fn gateway_addr(&self) -> Option<std::net::Ipv4Addr>;
}
