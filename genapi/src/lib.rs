#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::option_if_let_else,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

pub mod builder;
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
mod ivalue;
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
pub use store::{CacheStore, NodeId, NodeStore, ValueStore};
pub use string::StringNode;
pub use string_reg::StringRegNode;
pub use swiss_knife::SwissKnifeNode;

use std::borrow::Cow;

use auto_impl::auto_impl;
use tracing::error;

pub mod prelude {
    pub use super::interface::{
        IBoolean, ICategory, ICommand, IEnumeration, IFloat, IInteger, IPort, IRegister, ISelector,
        IString,
    };
}

#[auto_impl(&mut, Box)]
pub trait Device {
    type Error: std::error::Error + 'static;

    fn read_mem(&mut self, address: i64, buf: &mut [u8]) -> Result<(), Self::Error>;

    fn write_mem(&mut self, address: i64, data: &[u8]) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum GenApiError {
    #[error("device I/O error: {0}")]
    Device(Box<dyn std::error::Error>),

    /// Read/Write access to the `GenApi` node is denied.
    #[error("access is denied: {0}")]
    AccessDenied(Cow<'static, str>),

    /// Invalid node.
    #[error("invalid node: {0}")]
    InvalidNode(Cow<'static, str>),

    /// Try to write invalid value to the node.
    ///
    /// e.g. try to write the value that exceeds the max value of the node.
    #[error("invalid data: {0}")]
    InvalidData(Cow<'static, str>),

    /// Operation on the node failed due to the lack of chunk data where it's required to complete the operation.
    #[error("chunk data missing")]
    ChunkDataMissing,

    /// Invalid buffer.
    #[error("invalid buffer: {0}")]
    InvalidBuffer(Cow<'static, str>),
}

impl GenApiError {
    fn device(inner: Box<dyn std::error::Error>) -> Self {
        let err = GenApiError::Device(inner);
        error!("{}", err);
        err
    }

    fn access_denied(inner: Cow<'static, str>) -> Self {
        let err = GenApiError::AccessDenied(inner);
        error!("{}", err);
        err
    }

    fn invalid_node(inner: Cow<'static, str>) -> Self {
        let err = GenApiError::InvalidNode(inner);
        error!("{}", err);
        err
    }

    fn invalid_data(inner: Cow<'static, str>) -> Self {
        let err = GenApiError::InvalidData(inner);
        error!("{}", err);
        err
    }

    fn chunk_data_missing() -> Self {
        let err = GenApiError::ChunkDataMissing;
        error!("{}", err);
        err
    }

    fn invalid_buffer(inner: Cow<'static, str>) -> Self {
        let err = GenApiError::InvalidBuffer(inner);
        error!("{}", err);
        err
    }
}

pub type GenApiResult<T> = std::result::Result<T, GenApiError>;

#[derive(Clone, Debug)]
pub struct ValueCtxt<T, U> {
    pub value_store: T,
    pub cache_store: U,
}

impl<T, U> ValueCtxt<T, U> {
    pub fn new(value_store: T, cache_store: U) -> Self {
        Self {
            value_store,
            cache_store,
        }
    }

    pub fn value_store(&self) -> &T {
        &self.value_store
    }

    pub fn value_store_mut(&mut self) -> &mut T {
        &mut self.value_store
    }

    pub fn cache_store(&mut self) -> &U
    where
        T: store::ValueStore,
        U: store::CacheStore,
    {
        &self.cache_store
    }

    pub fn cache_store_mut(&mut self) -> &mut U {
        &mut self.cache_store
    }

    pub fn cache_data(&mut self, nid: store::NodeId, address: i64, length: i64, value: &[u8])
    where
        U: store::CacheStore,
    {
        self.cache_store.cache(nid, address, length, value);
    }

    pub fn get_cache(&self, nid: store::NodeId, address: i64, length: i64) -> Option<&[u8]>
    where
        U: store::CacheStore,
    {
        self.cache_store.get_cache(nid, address, length)
    }

    pub fn invalidate_cache_by(&mut self, nid: store::NodeId)
    where
        U: store::CacheStore,
    {
        self.cache_store.invalidate_by(nid)
    }

    pub fn invalidate_cache_of(&mut self, nid: store::NodeId)
    where
        U: store::CacheStore,
    {
        self.cache_store.invalidate_of(nid)
    }
}
