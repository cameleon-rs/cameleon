#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::option_if_let_else
)]

pub mod elem_type;
pub mod formula;
pub mod interface;
pub mod parser;
pub mod store;

mod boolean;
mod category;
mod command;
mod converter;
mod enumeration;
mod float;
mod float_reg;
mod int_converter;
mod int_reg;
mod int_swiss_knife;
mod integer;
mod masked_int_reg;
mod node;
mod node_base;
mod port;
mod register;
mod register_base;
mod register_description;
mod string;
mod string_reg;
mod swiss_knife;
mod utils;

pub use boolean::BooleanNode;
pub use category::CategoryNode;
pub use command::CommandNode;
pub use converter::ConverterNode;
pub use enumeration::{EnumEntryNode, EnumerationNode};
pub use float::FloatNode;
pub use float_reg::FloatRegNode;
pub use int_converter::IntConverterNode;
pub use int_reg::IntRegNode;
pub use int_swiss_knife::IntSwissKnifeNode;
pub use integer::IntegerNode;
pub use masked_int_reg::MaskedIntRegNode;
pub use node::Node;
pub use node_base::NodeBase;
pub use port::PortNode;
pub use register::RegisterNode;
pub use register_base::RegisterBase;
pub use register_description::RegisterDescription;
pub use string::StringNode;
pub use string_reg::StringRegNode;
pub use swiss_knife::SwissKnifeNode;

use std::borrow::Cow;

pub mod prelude {
    pub use super::interface::{
        IBoolean, ICategory, ICommand, IEnumeration, IFloat, IInteger, IPort, IRegister, ISelector,
        IString,
    };
}

pub trait Device {
    type Error: std::error::Error;

    fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> Result<(), Self::Error>;

    fn write_mem(&mut self, address: u64, data: &[u8]) -> Result<(), Self::Error>;
}

impl<T> Device for &mut T
where
    T: Device,
{
    type Error = T::Error;

    fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        (**self).read_mem(address, buf)
    }

    fn write_mem(&mut self, address: u64, data: &[u8]) -> Result<(), Self::Error> {
        (**self).write_mem(address, data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenApiError {
    /// Device I/O error.
    #[error("device I/O error: {}", 0)]
    Device(Box<dyn std::error::Error>),

    /// Read/Write access to the `GenApi` node is denied.
    #[error("access is denied: {}", 0)]
    AccessDenied(Cow<'static, str>),

    /// Invalid node.
    #[error("invalid node: {}", 0)]
    InvalidNode(Cow<'static, str>),

    /// Try to write invalid value to the node.
    ///
    /// e.g. try to write the value that exceeds the max value of the node.
    #[error("invalid data: {}", 0)]
    InvalidData(Cow<'static, str>),

    /// Operation on the node failed due to the lack of chunk data where it's required to complete the operation.
    #[error("chunk data missing")]
    ChunkDataMissing,

    /// Invalid buffer.
    #[error("invalid buffer: {}", 0)]
    InvalidBuffer(Cow<'static, str>),
}

pub type GenApiResult<T> = std::result::Result<T, GenApiError>;

#[derive(Clone, Debug)]
pub struct ValueCtxt<T, U>
where
    T: store::ValueStore,
    U: store::CacheStore,
{
    value_store: T,
    cache_store: U,
}

impl<T, U> ValueCtxt<T, U>
where
    T: store::ValueStore,
    U: store::CacheStore,
{
    pub fn value_store(&self) -> &T {
        &self.value_store
    }

    pub fn value_store_mut(&mut self) -> &mut T {
        &mut self.value_store
    }

    pub fn cache_store(&mut self) -> &U {
        &self.cache_store
    }

    pub fn cache_store_mut(&mut self) -> &mut U {
        &mut self.cache_store
    }

    pub fn cache_data(&mut self, nid: store::NodeId, value: impl Into<store::ValueData>) {
        if let Some(data) = self.cache_store.value(nid) {
            self.value_store.update(data.value_id, value);
            self.cache_store.store(nid, data.value_id)
        } else {
            let vid: store::ValueId = self.value_store.store(value);
            self.cache_store.store(nid, vid)
        }
    }

    pub fn get_cached(&self, nid: store::NodeId) -> Option<&store::ValueData> {
        let cache_data = self.cache_store.value(nid)?;
        self.value_store.value_opt(cache_data.value_id)
    }

    pub fn invalidate_cache_by(&mut self, nid: store::NodeId) {
        self.cache_store.invalidate_by(nid)
    }
}
