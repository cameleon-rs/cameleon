//! This module provides access to `GenApi` features of `GenICam` a compatible camera.
pub mod node_kind;

use std::sync::{Arc, Mutex};

use auto_impl::auto_impl;
use cameleon_genapi::{store, CacheStore, NodeStore, ValueCtxt, ValueStore};

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
    /// Enter context.
    pub fn enter<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Ctrl, &mut Ctxt) -> R,
    {
        f(&mut self.ctrl, &mut self.ctxt)
    }
}

/// The trait that provides accesss to `GenApi` context.
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
        F: FnMut(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R;
}

/// Default `GenApi` context.  
/// This context caches values of `GenApi` nodes if possible to reduce transaction.
///
/// If you need no cache context, use [`NoCacheGenApiCtxt`].
#[derive(Debug)]
pub struct DefaultGenApiCtxt {
    node_store: store::DefaultNodeStore,
    value_ctxt: ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>,
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
}

/// A sharable version of [`DefaultGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedDefaultGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>>>,
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
}

impl From<DefaultGenApiCtxt> for SharedDefaultGenApiCtxt {
    fn from(ctxt: DefaultGenApiCtxt) -> Self {
        Self {
            node_store: Arc::new(ctxt.node_store),
            value_ctxt: Arc::new(Mutex::new(ctxt.value_ctxt)),
        }
    }
}

/// `GenApi` context.  
/// This context doesn't cache any value of `GenApi` nodes.
#[derive(Debug)]
pub struct NoCacheGenApiCtxt {
    node_store: store::DefaultNodeStore,
    value_ctxt: ValueCtxt<store::DefaultValueStore, store::CacheSink>,
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
}

impl From<DefaultGenApiCtxt> for NoCacheGenApiCtxt {
    fn from(from: DefaultGenApiCtxt) -> Self {
        Self {
            node_store: from.node_store,
            value_ctxt: ValueCtxt::new(from.value_ctxt.value_store, store::CacheSink::default()),
        }
    }
}

/// A sharable version of [`NoCacheGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedNoCacheGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::CacheSink>>>,
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
}

impl From<NoCacheGenApiCtxt> for SharedNoCacheGenApiCtxt {
    fn from(from: NoCacheGenApiCtxt) -> Self {
        Self {
            node_store: Arc::new(from.node_store),
            value_ctxt: Arc::new(Mutex::new(from.value_ctxt)),
        }
    }
}

impl From<DefaultGenApiCtxt> for SharedNoCacheGenApiCtxt {
    fn from(from: DefaultGenApiCtxt) -> Self {
        let ctxt: NoCacheGenApiCtxt = from.into();
        ctxt.into()
    }
}
