use std::{collections::HashMap, convert::TryFrom};

use auto_impl::auto_impl;
use string_interner::{StringInterner, Symbol};

use super::{
    builder,
    interface::{
        IBooleanKind, ICategoryKind, ICommandKind, IEnumerationKind, IFloatKind, IIntegerKind,
        IPortKind, IRegisterKind, ISelectorKind, IStringKind,
    },
    node_base::NodeBase,
    BooleanNode, CategoryNode, CommandNode, ConverterNode, EnumerationNode, FloatNode,
    FloatRegNode, GenApiError, GenApiResult, IntConverterNode, IntRegNode, IntSwissKnifeNode,
    IntegerNode, MaskedIntRegNode, Node, PortNode, RegisterNode, StringNode, StringRegNode,
    SwissKnifeNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

#[derive(Debug, Clone)]
pub enum NodeData {
    Node(Box<Node>),
    Category(Box<CategoryNode>),
    Integer(Box<IntegerNode>),
    IntReg(Box<IntRegNode>),
    MaskedIntReg(Box<MaskedIntRegNode>),
    Boolean(Box<BooleanNode>),
    Command(Box<CommandNode>),
    Enumeration(Box<EnumerationNode>),
    Float(Box<FloatNode>),
    FloatReg(Box<FloatRegNode>),
    String(Box<StringNode>),
    StringReg(Box<StringRegNode>),
    Register(Box<RegisterNode>),
    Converter(Box<ConverterNode>),
    IntConverter(Box<IntConverterNode>),
    SwissKnife(Box<SwissKnifeNode>),
    IntSwissKnife(Box<IntSwissKnifeNode>),
    Port(Box<PortNode>),

    // TODO: Implement DCAM specific ndoes.
    ConfRom(()),
    TextDesc(()),
    IntKey(()),
    AdvFeatureLock(()),
    SmartFeature(()),
}

#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait NodeStore {
    fn name_by_id(&self, nid: NodeId) -> Option<&str>;

    fn id_by_name<T>(&self, s: T) -> Option<NodeId>
    where
        T: AsRef<str>;

    fn node_opt(&self, nid: NodeId) -> Option<&NodeData>;

    fn node(&self, nid: NodeId) -> &NodeData {
        self.node_opt(nid).unwrap()
    }

    fn visit_nodes<F>(&self, f: F)
    where
        F: FnMut(&NodeData);
}

#[auto_impl(&mut, Box)]
pub trait ValueStore {
    fn value_opt<T>(&self, id: T) -> Option<&ValueData>
    where
        T: Into<ValueId>;

    fn update<T, U>(&mut self, id: T, value: U) -> Option<ValueData>
    where
        T: Into<ValueId>,
        U: Into<ValueData>;

    fn value(&self, id: impl Into<ValueId>) -> &ValueData {
        self.value_opt(id).unwrap()
    }

    fn integer_value(&self, id: IntegerId) -> Option<i64> {
        if let ValueData::Integer(i) = self.value_opt(id)? {
            Some(*i)
        } else {
            None
        }
    }

    fn float_value(&self, id: FloatId) -> Option<f64> {
        if let ValueData::Float(f) = self.value_opt(id)? {
            Some(*f)
        } else {
            None
        }
    }

    fn str_value(&self, id: StringId) -> Option<&String> {
        if let ValueData::Str(s) = self.value_opt(id)? {
            Some(s)
        } else {
            None
        }
    }

    fn boolean_value(&self, id: BooleanId) -> Option<bool> {
        if let ValueData::Boolean(b) = self.value_opt(id)? {
            Some(*b)
        } else {
            None
        }
    }
}

#[auto_impl(&mut, Box)]
pub trait CacheStore {
    fn cache(&mut self, nid: NodeId, data: &[u8]);

    fn get_cache(&self, nid: NodeId) -> Option<&[u8]>;

    fn invalidate_by(&mut self, nid: NodeId);

    fn invalidate_of(&mut self, nid: NodeId);
}

impl Symbol for NodeId {
    fn try_from_usize(index: usize) -> Option<Self> {
        if ((u32::MAX - 1) as usize) < index {
            None
        } else {
            #[allow(clippy::cast_possible_truncation)]
            Some(Self(index as u32))
        }
    }

    fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl NodeId {
    pub fn name<'a>(self, store: &'a impl NodeStore) -> &'a str {
        store.name_by_id(self).unwrap()
    }

    pub fn as_iinteger_kind<'a>(self, store: &'a impl NodeStore) -> Option<IIntegerKind<'a>> {
        IIntegerKind::maybe_from(self, store)
    }

    pub fn expect_iinteger_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<IIntegerKind<'a>> {
        self.as_iinteger_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `IInteger`".into(),
            ))
    }

    pub fn as_ifloat_kind<'a>(self, store: &'a impl NodeStore) -> Option<IFloatKind<'a>> {
        IFloatKind::maybe_from(self, store)
    }

    pub fn expect_ifloat_kind<'a>(self, store: &'a impl NodeStore) -> GenApiResult<IFloatKind<'a>> {
        self.as_ifloat_kind(store).ok_or(GenApiError::invalid_node(
            "the node doesn't implement `IFloat`".into(),
        ))
    }

    pub fn as_istring_kind<'a>(self, store: &'a impl NodeStore) -> Option<IStringKind<'a>> {
        IStringKind::maybe_from(self, store)
    }

    pub fn expect_istring_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<IStringKind<'a>> {
        self.as_istring_kind(store).ok_or(GenApiError::invalid_node(
            "the node doesn't implement `IString`".into(),
        ))
    }

    pub fn as_icommand_kind<'a>(self, store: &'a impl NodeStore) -> Option<ICommandKind<'a>> {
        ICommandKind::maybe_from(self, store)
    }

    pub fn expect_icommand_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<ICommandKind<'a>> {
        self.as_icommand_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `ICommand`".into(),
            ))
    }

    pub fn as_ienumeration_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> Option<IEnumerationKind<'a>> {
        IEnumerationKind::maybe_from(self, store)
    }

    pub fn expect_ienumeration_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<IEnumerationKind<'a>> {
        self.as_ienumeration_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `IEnumeration`".into(),
            ))
    }

    pub fn as_iboolean_kind<'a>(self, store: &'a impl NodeStore) -> Option<IBooleanKind<'a>> {
        IBooleanKind::maybe_from(self, store)
    }

    pub fn expect_iboolean_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<IBooleanKind<'a>> {
        self.as_iboolean_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `IBoolean`".into(),
            ))
    }

    pub fn as_iregister_kind<'a>(self, store: &'a impl NodeStore) -> Option<IRegisterKind<'a>> {
        IRegisterKind::maybe_from(self, store)
    }

    pub fn expect_iregister_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<IRegisterKind<'a>> {
        self.as_iregister_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `IRegister`".into(),
            ))
    }

    pub fn as_icategory_kind<'a>(self, store: &'a impl NodeStore) -> Option<ICategoryKind<'a>> {
        ICategoryKind::maybe_from(self, store)
    }

    pub fn expect_icategory_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<ICategoryKind<'a>> {
        self.as_icategory_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `ICategory`".into(),
            ))
    }

    pub fn as_iport_kind<'a>(self, store: &'a impl NodeStore) -> Option<IPortKind<'a>> {
        IPortKind::maybe_from(self, store)
    }

    pub fn expect_iport_kind<'a>(self, store: &'a impl NodeStore) -> GenApiResult<IPortKind<'a>> {
        self.as_iport_kind(store).ok_or(GenApiError::invalid_node(
            "the node doesn't implement `IPort`".into(),
        ))
    }

    pub fn as_iselector_kind<'a>(self, store: &'a impl NodeStore) -> Option<ISelectorKind<'a>> {
        ISelectorKind::maybe_from(self, store)
    }

    pub fn expect_iselector_kind<'a>(
        self,
        store: &'a impl NodeStore,
    ) -> GenApiResult<ISelectorKind<'a>> {
        self.as_iselector_kind(store)
            .ok_or(GenApiError::invalid_node(
                "the node doesn't implement `ISelector`".into(),
            ))
    }
}

impl NodeData {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        match self {
            Self::Node(node) => node.node_base(),
            Self::Category(node) => node.node_base(),
            Self::Integer(node) => node.node_base(),
            Self::IntReg(node) => node.node_base(),
            Self::MaskedIntReg(node) => node.node_base(),
            Self::Boolean(node) => node.node_base(),
            Self::Command(node) => node.node_base(),
            Self::Enumeration(node) => node.node_base(),
            Self::Float(node) => node.node_base(),
            Self::FloatReg(node) => node.node_base(),
            Self::String(node) => node.node_base(),
            Self::StringReg(node) => node.node_base(),
            Self::Register(node) => node.node_base(),
            Self::Converter(node) => node.node_base(),
            Self::IntConverter(node) => node.node_base(),
            Self::SwissKnife(node) => node.node_base(),
            Self::IntSwissKnife(node) => node.node_base(),
            Self::Port(node) => node.node_base(),
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct DefaultNodeStore {
    pub(super) interner: StringInterner<NodeId>,
    pub(super) store: Vec<Option<NodeData>>,
}

impl DefaultNodeStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            store: Vec::new(),
        }
    }
}

impl NodeStore for DefaultNodeStore {
    fn name_by_id(&self, nid: NodeId) -> Option<&str> {
        self.interner.resolve(nid)
    }

    fn id_by_name<T>(&self, s: T) -> Option<NodeId>
    where
        T: AsRef<str>,
    {
        self.interner.get(s)
    }

    fn node_opt(&self, nid: NodeId) -> Option<&NodeData> {
        self.store.get(nid.to_usize())?.as_ref()
    }

    fn visit_nodes<F>(&self, mut f: F)
    where
        F: FnMut(&NodeData),
    {
        for data in &self.store {
            if let Some(data) = data {
                f(data);
            }
        }
    }
}

impl builder::NodeStoreBuilder for DefaultNodeStore {
    type Store = Self;

    fn build(self) -> Self {
        self
    }

    fn get_or_intern<T>(&mut self, s: T) -> NodeId
    where
        T: AsRef<str>,
    {
        self.interner.get_or_intern(s)
    }

    fn store_node(&mut self, nid: NodeId, data: NodeData) {
        let id = nid.to_usize();
        if self.store.len() <= id {
            self.store.resize(id + 1, None)
        }
        debug_assert!(self.store[id].is_none());
        self.store[id] = Some(data);
    }
}

impl Default for DefaultNodeStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(u32);

impl ValueId {
    #[must_use]
    pub fn from_u32(i: u32) -> Self {
        Self(i)
    }
}

macro_rules! declare_value_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(u32);

        impl From<$name> for ValueId {
            fn from(v: $name) -> ValueId {
                ValueId(v.0)
            }
        }

        impl From<ValueId> for $name {
            fn from(vid: ValueId) -> Self {
                Self(vid.0)
            }
        }
    };
}

declare_value_id!(IntegerId);
declare_value_id!(FloatId);
declare_value_id!(StringId);
declare_value_id!(BooleanId);

#[derive(Debug, Clone, PartialEq)]
pub enum ValueData {
    Integer(i64),
    Float(f64),
    Str(String),
    Boolean(bool),
}

macro_rules! impl_value_data_conversion {
    ($ty:ty, $ctor:expr) => {
        impl From<$ty> for ValueData {
            fn from(v: $ty) -> Self {
                $ctor(v)
            }
        }
    };
}

impl_value_data_conversion!(i64, Self::Integer);
impl_value_data_conversion!(f64, Self::Float);
impl_value_data_conversion!(String, Self::Str);
impl_value_data_conversion!(bool, Self::Boolean);

#[derive(Debug, Default)]
pub struct DefaultValueStore(Vec<ValueData>);

impl DefaultValueStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl builder::ValueStoreBuilder for DefaultValueStore {
    type Store = Self;

    fn build(self) -> Self {
        self
    }

    fn store<T, U>(&mut self, data: T) -> U
    where
        T: Into<ValueData>,
        U: From<ValueId>,
    {
        let id = u32::try_from(self.0.len())
            .expect("the number of value stored in `ValueStore` must not exceed u32::MAX");
        let id = ValueId(id);
        self.0.push(data.into());
        id.into()
    }
}

impl ValueStore for DefaultValueStore {
    fn value_opt<T>(&self, id: T) -> Option<&ValueData>
    where
        T: Into<ValueId>,
    {
        self.0.get(id.into().0 as usize)
    }

    fn update<T, U>(&mut self, id: T, value: U) -> Option<ValueData>
    where
        T: Into<ValueId>,
        U: Into<ValueData>,
    {
        self.0
            .get_mut(id.into().0 as usize)
            .map(|old| std::mem::replace(old, value.into()))
    }
}

#[derive(Debug, Default)]
pub struct DefaultCacheStore {
    store: HashMap<NodeId, Option<Vec<u8>>>,
    invalidators: HashMap<NodeId, Vec<NodeId>>,
}

impl DefaultCacheStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl builder::CacheStoreBuilder for DefaultCacheStore {
    type Store = Self;

    fn build(self) -> Self {
        self
    }

    fn store_invalidator(&mut self, invalidator: NodeId, target: NodeId) {
        let entry = self.invalidators.entry(invalidator).or_default();
        entry.push(target)
    }
}

impl CacheStore for DefaultCacheStore {
    fn cache(&mut self, nid: NodeId, data: &[u8]) {
        self.store
            .entry(nid)
            .and_modify(|cache| *cache = Some(data.to_vec()))
            .or_insert(Some(data.to_owned()));
    }

    fn get_cache(&self, nid: NodeId) -> Option<&[u8]> {
        self.store.get(&nid)?.as_deref()
    }

    fn invalidate_by(&mut self, nid: NodeId) {
        if let Some(target_nodes) = self.invalidators.get(&nid) {
            for nid in target_nodes {
                if let Some(cache) = self.store.get_mut(&nid) {
                    *cache = None;
                }
            }
        }
    }

    fn invalidate_of(&mut self, nid: NodeId) {
        if let Some(cache) = self.store.get_mut(&nid) {
            *cache = None;
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct CacheSink {
    _priv: (),
}

impl CacheSink {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl builder::CacheStoreBuilder for CacheSink {
    type Store = Self;

    fn build(self) -> Self {
        self
    }

    /// Store invalidator and its target to be invalidated.
    fn store_invalidator(&mut self, _: NodeId, _: NodeId) {}
}

impl CacheStore for CacheSink {
    fn cache(&mut self, _: NodeId, _: &[u8]) {}

    fn get_cache(&self, _: NodeId) -> Option<&[u8]> {
        None
    }

    fn invalidate_by(&mut self, _: NodeId) {}

    fn invalidate_of(&mut self, _: NodeId) {}
}
