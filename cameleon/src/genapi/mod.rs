//! This module provides access to `GenApi` features of `GenICam` a compatible camera.
pub mod node_kind;

use std::sync::{Arc, Mutex};

use cameleon_genapi::{
    builder::GenApiBuilder, store, CacheStore, NodeStore, RegisterDescription, ValueCtxt,
    ValueStore,
};

use super::{ControlError, ControlResult};

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
    pub fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Ctrl, &mut Ctxt) -> R,
    {
        f(&mut self.ctrl, &mut self.ctxt)
    }

    /// Enters the context and then enters `GenApiCtxt`.
    pub fn enter2<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Ctrl, &Ctxt::NS, &mut ValueCtxt<Ctxt::VS, Ctxt::CS>) -> R,
    {
        self.enter(|ctrl, ctxt| {
            ctxt.enter(|node_store, value_ctxt| f(ctrl, node_store, value_ctxt))
        })
    }
}

/// The trait that provides accesss to `GenApi` context.
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
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R;

    /// Parses `GenApi` xml and builds the context.
    fn build(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized;
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

    fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }

    fn build(xml: &impl AsRef<str>) -> ControlResult<Self> {
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

    fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
    }

    fn build(xml: &impl AsRef<str>) -> ControlResult<Self> {
        Ok(DefaultGenApiCtxt::build(xml)?.into())
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

    fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }

    fn build(xml: &impl AsRef<str>) -> ControlResult<Self> {
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

    fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
    }

    fn build(xml: &impl AsRef<str>) -> ControlResult<Self> {
        Ok(NoCacheGenApiCtxt::build(xml)?.into())
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
