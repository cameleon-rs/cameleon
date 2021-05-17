//! This module provides access to `GenApi` features of `GenICam` a compatible camera.
pub mod node_kind;

use std::sync::{Arc, Mutex};

use auto_impl::auto_impl;
use cameleon_genapi::{builder::GenApiBuilder, store};

use super::{ControlError, ControlResult};
use node_kind::Node;

pub use cameleon_genapi::{
    elem_type::{AccessMode, NameSpace, Visibility},
    store::{
        CacheSink, CacheStore, DefaultCacheStore, DefaultNodeStore, DefaultValueStore, NodeId,
        NodeStore, ValueStore,
    },
    RegisterDescription, ValueCtxt,
};

/// Manages context of parameters of the device.
#[derive(Debug, Clone)]
pub struct ParamsCtxt<Ctrl, Ctxt> {
    /// Control handle of the device.
    pub ctrl: Ctrl,
    /// `GenApi` context of the device.
    pub ctxt: Ctxt,
}

impl<Ctrl, Ctxt> ParamsCtxt<Ctrl, Ctxt>
where
    Ctrl: cameleon_genapi::Device,
    Ctxt: GenApiCtxt,
{
    /// Enters the context.
    pub fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Ctrl, &mut Ctxt) -> R,
    {
        f(&mut self.ctrl, &mut self.ctxt)
    }

    /// Enters the context and then enters `GenApiCtxt`.
    pub fn enter2<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Ctrl, &Ctxt::NS, &mut ValueCtxt<Ctxt::VS, Ctxt::CS>) -> R,
    {
        self.enter(|ctrl, ctxt| {
            ctxt.enter(|node_store, value_ctxt| f(ctrl, node_store, value_ctxt))
        })
    }
}

impl<Ctrl, Ctxt> ParamsCtxt<Ctrl, Ctxt>
where
    Ctxt: GenApiCtxt,
{
    /// Returns [`NodeStore`] in the context.
    pub fn node_store(&self) -> &Ctxt::NS {
        self.ctxt.node_store()
    }

    /// Returns Some(...) if there is a node with the given name in the context.
    pub fn node(&self, name: &str) -> Option<Node> {
        let ns = self.ctxt.node_store();
        ns.id_by_name(name).map(Node)
    }
}

impl<Ctrl, Ctxt> ParamsCtxt<Ctrl, Ctxt> {
    /// Converts internal types. This method work same as `std::convert::From`, just hack to avoid
    /// `E0119`.
    pub fn convert_from<Ctrl2, Ctxt2>(from: ParamsCtxt<Ctrl2, Ctxt2>) -> Self
    where
        Ctrl: From<Ctrl2>,
        Ctxt: From<Ctxt2>,
    {
        ParamsCtxt {
            ctrl: from.ctrl.into(),
            ctxt: from.ctxt.into(),
        }
    }

    /// Converts internal types. This method work same as `std::convert::Into`, just hack to avoid
    /// `E0119`.
    pub fn convert_into<Ctrl2, Ctxt2>(self) -> ParamsCtxt<Ctrl2, Ctxt2>
    where
        Ctrl: Into<Ctrl2>,
        Ctxt: Into<Ctxt2>,
    {
        ParamsCtxt {
            ctrl: self.ctrl.into(),
            ctxt: self.ctxt.into(),
        }
    }
}

/// A trait that provides accesss to `GenApi` context.
#[auto_impl(&mut, Box)]
pub trait GenApiCtxt {
    /// A type that implements [`NodeStore`]
    type NS: NodeStore;
    /// A type that implements [`ValueStore`]
    type VS: ValueStore;
    /// A type that implements [`CacheStore`]
    type CS: CacheStore;

    /// Provide access to the context.
    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R;

    /// Returns [`NodeStore`] in the context.
    fn node_store(&self) -> &Self::NS;

    /// Returns description of the `GenApi` xml.
    fn description(&self) -> &RegisterDescription;

    /// Clear all cache of the context.
    fn clear_cache(&mut self) {
        self.enter(|_, value_ctxt| value_ctxt.clear_cache())
    }
}

/// A trait that provides directly conversion from `GenApi` string to a `GenApi` context.
pub trait FromXml {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt;
}

/// Default `GenApi` context.  
/// This context caches values of `GenApi` nodes if possible to reduce transaction.
///
/// If you need no cache context, use [`NoCacheGenApiCtxt`].
#[derive(Debug)]
pub struct DefaultGenApiCtxt {
    node_store: store::DefaultNodeStore,
    value_ctxt: ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>,
    reg_desc: RegisterDescription,
}

impl GenApiCtxt for DefaultGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::DefaultCacheStore;

    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    fn description(&self) -> &RegisterDescription {
        &self.reg_desc
    }
}

impl FromXml for DefaultGenApiCtxt {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        let (reg_desc, node_store, value_ctxt) = GenApiBuilder::default()
            .build(xml)
            .map_err(|e| ControlError::InvalidData(e.into()))?;
        Ok(Self {
            node_store,
            value_ctxt,
            reg_desc,
        })
    }
}

/// A sharable version of [`DefaultGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedDefaultGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>>>,
    reg_desc: Arc<RegisterDescription>,
}

impl GenApiCtxt for SharedDefaultGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::DefaultCacheStore;

    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
    }

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    fn description(&self) -> &RegisterDescription {
        &self.reg_desc
    }
}

impl FromXml for SharedDefaultGenApiCtxt {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        Ok(DefaultGenApiCtxt::from_xml(xml)?.into())
    }
}

impl From<DefaultGenApiCtxt> for SharedDefaultGenApiCtxt {
    fn from(ctxt: DefaultGenApiCtxt) -> Self {
        Self {
            node_store: Arc::new(ctxt.node_store),
            value_ctxt: Arc::new(Mutex::new(ctxt.value_ctxt)),
            reg_desc: Arc::new(ctxt.reg_desc),
        }
    }
}

/// `GenApi` context.  
/// This context doesn't cache any value of `GenApi` nodes.
#[derive(Debug)]
pub struct NoCacheGenApiCtxt {
    node_store: store::DefaultNodeStore,
    value_ctxt: ValueCtxt<store::DefaultValueStore, store::CacheSink>,
    reg_desc: RegisterDescription,
}

impl GenApiCtxt for NoCacheGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::CacheSink;

    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    fn description(&self) -> &RegisterDescription {
        &self.reg_desc
    }
}

impl FromXml for NoCacheGenApiCtxt {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        let (reg_desc, node_store, value_ctxt) = GenApiBuilder::default()
            .no_cache()
            .build(xml)
            .map_err(|e| ControlError::InvalidData(e.into()))?;
        Ok(Self {
            node_store,
            value_ctxt,
            reg_desc,
        })
    }
}

impl From<DefaultGenApiCtxt> for NoCacheGenApiCtxt {
    fn from(from: DefaultGenApiCtxt) -> Self {
        Self {
            node_store: from.node_store,
            value_ctxt: ValueCtxt::new(from.value_ctxt.value_store, store::CacheSink::default()),
            reg_desc: from.reg_desc,
        }
    }
}

/// A sharable version of [`NoCacheGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedNoCacheGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::CacheSink>>>,
    reg_desc: Arc<RegisterDescription>,
}

impl GenApiCtxt for SharedNoCacheGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::CacheSink;

    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
    }

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    fn description(&self) -> &RegisterDescription {
        &self.reg_desc
    }
}

impl FromXml for SharedNoCacheGenApiCtxt {
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        Ok(NoCacheGenApiCtxt::from_xml(xml)?.into())
    }
}

impl From<NoCacheGenApiCtxt> for SharedNoCacheGenApiCtxt {
    fn from(from: NoCacheGenApiCtxt) -> Self {
        Self {
            node_store: Arc::new(from.node_store),
            value_ctxt: Arc::new(Mutex::new(from.value_ctxt)),
            reg_desc: Arc::new(from.reg_desc),
        }
    }
}

impl From<DefaultGenApiCtxt> for SharedNoCacheGenApiCtxt {
    fn from(from: DefaultGenApiCtxt) -> Self {
        let ctxt: NoCacheGenApiCtxt = from.into();
        ctxt.into()
    }
}

/// Represents `CompressionType` of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    /// Uncompressed `GenICam` XML file.
    Uncompressed,
    /// ZIP containing a single `GenICam` XML file.
    Zip,
}
