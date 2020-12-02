use std::sync::Mutex;

use crate::{imp::device::Device, imp::port::*, GenTlResult};

pub(crate) mod u3v;

mod u3v_memory;

pub(crate) trait Interface: Port {
    fn open(&mut self) -> GenTlResult<()>;

    fn close(&mut self) -> GenTlResult<()>;

    fn interface_id(&self) -> &str;

    fn display_name(&self) -> &str;

    fn tl_type(&self) -> TlType;

    fn mac_addr(&self) -> Option<[u8; 6]>;

    fn ip_addr(&self) -> Option<std::net::Ipv4Addr>;

    fn subnet_mask(&self) -> Option<std::net::Ipv4Addr>;

    fn gateway_addr(&self) -> Option<std::net::Ipv4Addr>;

    fn devices(&self) -> Vec<&Mutex<dyn Device>>;
}
