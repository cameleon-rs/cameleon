/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module provides access to `GenApi` features of `GenICam` a compatible camera.
//!
//! # Examples
//! ```rust
//! # use cameleon::u3v;
//! # let mut cameras = u3v::enumerate_cameras().unwrap();
//! # if cameras.is_empty() {
//! #     return;
//! # }
//! # let mut camera = cameras.pop().unwrap();
//! // Loads `GenApi` context.
//! camera.load_context().unwrap();
//!
//! let mut params_ctxt = camera.params_ctxt().unwrap();
//! // Get `Gain` node of `GenApi`.
//! // `GenApi SFNC` defines that `Gain` node should have `IFloat` interface,
//! // so this conversion would be success if the camera follows that.
//! // Some vendors may define `Gain` node as `IInteger`, in that case, use
//! // `as_integer(&params_ctxt)` instead of `as_float(&params_ctxt).
//! let gain_node = params_ctxt.node("Gain").unwrap().as_float(&params_ctxt).unwrap();
//!
//! // Get the current value of `Gain`.
//! if gain_node.is_readable(&mut params_ctxt).unwrap() {
//!     let value = gain_node.value(&mut params_ctxt).unwrap();
//!     println!("{}", value);
//! }
//!
//! // Set `0.1` to `Gain`.
//! if gain_node.is_writable(&mut params_ctxt).unwrap() {
//!     gain_node.set_value(&mut params_ctxt, 0.1).unwrap();
//! }
//! ```

mod node_kind;

pub use node_kind::{
    BooleanNode, CategoryNode, CommandNode, EnumEntryNode, EnumerationNode, FloatNode, IntegerNode,
    Node, PortNode, RegisterNode, StringNode,
};

use std::{
    convert::TryInto,
    sync::{Arc, Mutex},
};

use auto_impl::auto_impl;
use cameleon_genapi::{builder::GenApiBuilder, store};

use super::{ControlError, ControlResult, DeviceControl};

pub use cameleon_genapi::{
    elem_type::{AccessMode, NameSpace, Visibility},
    store::{
        CacheSink, CacheStore, DefaultCacheStore, DefaultNodeStore, DefaultValueStore, NodeId,
        NodeStore, ValueStore,
    },
    GenApiError, RegisterDescription, ValueCtxt,
};

/// Manages context of parameters of the device.
///
/// # Examples
/// ```rust
/// # use cameleon::u3v;
/// # let mut cameras = u3v::enumerate_cameras().unwrap();
/// # if cameras.is_empty() {
/// #     return;
/// # }
/// # let mut camera = cameras.pop().unwrap();
/// // Loads `GenApi` context.
/// camera.load_context().unwrap();
///
/// let mut params_ctxt = camera.params_ctxt().unwrap();
/// // Get `Gain` node of `GenApi`.
/// // `GenApi SFNC` defines that `Gain` node should have `IFloat` interface,
/// // so this conversion would be success if the camera follows that.
/// // Some vendors may define `Gain` node as `IInteger`, in that case, use
/// // `as_integer(&params_ctxt)` instead of `as_float(&params_ctxt).
/// let gain_node = params_ctxt.node("Gain").unwrap().as_float(&params_ctxt).unwrap();
///
/// // Get the current value of `Gain`.
/// if gain_node.is_readable(&mut params_ctxt).unwrap() {
///     let value = gain_node.value(&mut params_ctxt).unwrap();
///     println!("{}", value);
/// }
///
/// // Set `0.1` to `Gain`.
/// if gain_node.is_writable(&mut params_ctxt).unwrap() {
///     gain_node.set_value(&mut params_ctxt, 0.1).unwrap();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ParamsCtxt<Ctrl, Ctxt> {
    /// Control handle of the device.
    pub ctrl: Ctrl,
    /// `GenApi` context of the device.
    pub ctxt: Ctxt,
}

impl<Ctrl, Ctxt> ParamsCtxt<Ctrl, Ctxt>
where
    Ctxt: GenApiCtxt,
{
    /// Returns `None` if there is node node with the given name in the context.
    pub fn node(&self, name: &str) -> Option<Node> {
        let ns = self.ctxt.node_store();
        ns.id_by_name(name).map(Node)
    }

    /// Returns [`NodeStore`] in the context.
    pub fn node_store(&self) -> &Ctxt::NS {
        self.ctxt.node_store()
    }
}

impl<Ctrl, Ctxt> ParamsCtxt<Ctrl, Ctxt>
where
    Ctrl: DeviceControl,
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
    /// Node store.
    pub node_store: store::DefaultNodeStore,
    /// Value context.
    pub value_ctxt: ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>,
    /// Register description.
    pub reg_desc: RegisterDescription,
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
}

impl FromXml for DefaultGenApiCtxt {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        let (reg_desc, node_store, value_ctxt) = GenApiBuilder::<DefaultNodeStore>::default()
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
    /// Node store.
    pub node_store: Arc<store::DefaultNodeStore>,
    /// Value context.
    pub value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::DefaultCacheStore>>>,
    /// Register description.
    pub reg_desc: Arc<RegisterDescription>,
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
    /// Node store.
    pub node_store: store::DefaultNodeStore,
    /// Value context.
    pub value_ctxt: ValueCtxt<store::DefaultValueStore, store::CacheSink>,
    /// Register description.
    pub reg_desc: RegisterDescription,
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
}

impl FromXml for NoCacheGenApiCtxt {
    /// Parse `GenApi` context and build `
    fn from_xml(xml: &impl AsRef<str>) -> ControlResult<Self>
    where
        Self: Sized + GenApiCtxt,
    {
        let (reg_desc, node_store, value_ctxt) = GenApiBuilder::<DefaultNodeStore>::default()
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
    /// Node store.
    pub node_store: Arc<store::DefaultNodeStore>,
    /// Value context.
    pub value_ctxt: Arc<Mutex<ValueCtxt<store::DefaultValueStore, store::CacheSink>>>,
    /// Register description.
    pub reg_desc: Arc<RegisterDescription>,
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

struct GenApiDevice<'a, T> {
    inner: &'a mut T,
}

impl<'a, T> GenApiDevice<'a, T> {
    fn new(inner: &'a mut T) -> Self {
        Self { inner }
    }
}

impl<'a, T> cameleon_genapi::Device for GenApiDevice<'a, T>
where
    T: DeviceControl,
{
    fn read_mem(
        &mut self,
        address: i64,
        data: &mut [u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let address: u64 = address.try_into().map_err(|_| {
            ControlError::InvalidData(
                "invalid address: the given address has negative value".into(),
            )
        })?;
        Ok(self.inner.read(address, data)?)
    }

    fn write_mem(
        &mut self,
        address: i64,
        data: &[u8],
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let address: u64 = address.try_into().map_err(|_| {
            ControlError::InvalidData(
                "invalid address: the given address has negative value".into(),
            )
        })?;
        Ok(self.inner.write(address, data)?)
    }
}
