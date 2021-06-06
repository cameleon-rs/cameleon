//! This example describes how to define custom `GenApi` context.
//!
//! In this example, we'll define a context in which the cache can be dynamically switched on and off.

use cameleon::genapi::{
    CacheStore, DefaultCacheStore, DefaultGenApiCtxt, DefaultNodeStore, DefaultValueStore,
    GenApiCtxt, NodeId, ValueCtxt,
};
use cameleon::{u3v, Camera};

/// Step1: Define `MyCacheStore` and implement `CacheStore` for it.
struct MyCacheStore {
    store: DefaultCacheStore,
    use_cache: bool,
}
impl CacheStore for MyCacheStore {
    fn cache(&mut self, nid: NodeId, address: i64, length: i64, data: &[u8]) {
        if self.use_cache {
            self.store.cache(nid, address, length, data)
        }
    }
    fn get_cache(&self, nid: NodeId, address: i64, length: i64) -> Option<&[u8]> {
        if self.use_cache {
            self.store.get_cache(nid, address, length)
        } else {
            None
        }
    }
    fn invalidate_by(&mut self, nid: NodeId) {
        if self.use_cache {
            self.store.invalidate_by(nid)
        }
    }
    fn invalidate_of(&mut self, nid: NodeId) {
        if self.use_cache {
            self.store.invalidate_of(nid)
        }
    }
    fn clear(&mut self) {
        if self.use_cache {
            self.store.clear();
        }
    }
}

/// Step2: Define `MyGenApiCtxt` and implement `GenApiCtxt` for it.
/// We use `DefaultNodeStore` and `DefaultValueStore` for simplicity.
struct MyGenApiCtxt {
    node_store: DefaultNodeStore,
    value_ctxt: ValueCtxt<DefaultValueStore, MyCacheStore>,
}
impl GenApiCtxt for MyGenApiCtxt {
    type NS = DefaultNodeStore;
    type VS = DefaultValueStore;
    type CS = MyCacheStore;

    fn enter<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&Self::NS, &mut ValueCtxt<Self::VS, Self::CS>) -> R,
    {
        f(&self.node_store, &mut self.value_ctxt)
    }

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    fn clear_cache(&mut self) {
        self.value_ctxt.clear_cache()
    }
}

/// Step3: Add utility methods to switch whether the context use the cache or not.
impl MyGenApiCtxt {
    /// Enable the context to use cache.
    fn enable_cache(&mut self) {
        self.value_ctxt.cache_store.use_cache = true;
    }

    /// Disable the context to use cache.
    fn disable_cache(&mut self) {
        self.value_ctxt.cache_store.use_cache = false;
    }
}

/// Step4: Implement `From<DefaultGenApiCtxt>` for `MyGenApiCtxt` to enable to use
/// `Camera::convert_into`.
impl From<DefaultGenApiCtxt> for MyGenApiCtxt {
    fn from(from: DefaultGenApiCtxt) -> Self {
        let value_ctxt = ValueCtxt::new(
            from.value_ctxt.value_store,
            MyCacheStore {
                store: from.value_ctxt.cache_store,
                use_cache: true,
            },
        );
        Self {
            node_store: from.node_store,
            value_ctxt,
        }
    }
}

fn main() {
    let mut cameras = u3v::enumerate_cameras().unwrap();
    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let mut camera = cameras.pop().unwrap();
    camera.open().unwrap();
    camera.load_context().unwrap();

    // Convert `DefaultGenApiCtxt` into our `MyGenApiCtxt`.
    let mut camera: Camera<u3v::ControlHandle, u3v::StreamHandle, MyGenApiCtxt> =
        camera.convert_into();
    let mut params_ctxt = camera.params_ctxt().unwrap();

    // Enable cache.
    params_ctxt.ctxt.enable_cache();
    let gain_node = params_ctxt
        .node("Gain")
        .unwrap()
        .as_float(&params_ctxt)
        .unwrap();
    gain_node.set_value(&mut params_ctxt, 100.1).unwrap();

    // Disable cache.
    params_ctxt.ctxt.disable_cache();
    // This call will read the device's memory even if its cache exists in the context.
    let value = gain_node.value(&mut params_ctxt).unwrap();
    println!("Gain: {}", value);

    camera.close().unwrap();
}
