//! This module provides access to `GenApi` features of `GenICam` a compatible camera.
use std::sync::{Arc, Mutex};

use cameleon_genapi::{store, CacheStore, NodeStore, ValueCtxt, ValueStore};

/// The trait that provides accesss to `GenApi` context.
pub trait GenApiCtxt {
    /// A type that implements [`NodeStore`]
    type NS: NodeStore;
    /// A type that implements [`ValueStore`]
    type VS: ValueStore;
    /// A type that implements [`CacheStore`]
    type CS: CacheStore;

    /// Provide access to the context.
    fn with_ctxt<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R;
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

    fn with_ctxt<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }
}

/// Sharable version of [`DefaultGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedDefaultGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>>>,
}

impl GenApiCtxt for SharedDefaultGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::DefaultCacheStore;

    fn with_ctxt<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
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

    fn with_ctxt<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }
}

/// Sharable version of [`NoCacheGenApiCtxt`].
#[derive(Clone, Debug)]
pub struct SharedNoCacheGenApiCtxt {
    node_store: Arc<store::DefaultNodeStore>,
    value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::CacheSink>>>,
}

impl GenApiCtxt for SharedNoCacheGenApiCtxt {
    type NS = store::DefaultNodeStore;
    type VS = store::DefaultValueStore;
    type CS = store::CacheSink;

    fn with_ctxt<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt.lock().unwrap())
    }
}
