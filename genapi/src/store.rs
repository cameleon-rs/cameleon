use std::convert::TryFrom;

use string_interner::{StringInterner, Symbol};

use super::{
    node_base::NodeBase, BooleanNode, CategoryNode, CommandNode, ConverterNode, EnumerationNode,
    FloatNode, FloatRegNode, IntConverterNode, IntRegNode, IntSwissKnifeNode, IntegerNode,
    MaskedIntRegNode, Node, PortNode, RegisterNode, StringNode, StringRegNode, SwissKnifeNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u32);

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

pub trait NodeStore {
    fn id_by_name<T: AsRef<str>>(&mut self, s: T) -> NodeId;

    fn node_opt(&self, id: NodeId) -> Option<&NodeData>;

    fn node(&self, id: NodeId) -> &NodeData {
        self.node_opt(id).unwrap()
    }

    fn store_node(&mut self, id: NodeId, data: NodeData);
}

impl<T> NodeStore for &mut T
where
    T: NodeStore,
{
    fn id_by_name<U: AsRef<str>>(&mut self, s: U) -> NodeId {
        (*self).id_by_name(s)
    }

    fn node_opt(&self, id: NodeId) -> Option<&NodeData> {
        (**self).node_opt(id)
    }

    fn store_node(&mut self, id: NodeId, data: NodeData) {
        (*self).store_node(id, data)
    }
}

pub struct DefaultNodeStore {
    interner: StringInterner<NodeId>,
    store: Vec<Option<NodeData>>,
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
    fn id_by_name<T: AsRef<str>>(&mut self, s: T) -> NodeId {
        self.interner.get_or_intern(s)
    }

    fn node_opt(&self, id: NodeId) -> Option<&NodeData> {
        self.store.get(id.to_usize())?.as_ref()
    }

    fn store_node(&mut self, id: NodeId, data: NodeData) {
        let id = id.to_usize();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueId(u32);

impl ValueId {
    pub fn from_u32(i: u32) -> Self {
        Self(i)
    }
}

macro_rules! declare_value_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(u32);

        impl Into<ValueId> for $name {
            fn into(self) -> ValueId {
                ValueId(self.0)
            }
        }

        impl From<ValueId> for $name {
            fn from(id: ValueId) -> Self {
                Self(id.0)
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

pub trait ValueStore {
    fn store<T, U>(&mut self, data: T) -> U
    where
        T: Into<ValueData>,
        U: From<ValueId>;

    fn value_opt(&self, id: impl Into<ValueId>) -> Option<&ValueData>;

    fn value_mut_opt(&mut self, id: impl Into<ValueId>) -> Option<&mut ValueData>;

    fn value(&self, id: impl Into<ValueId>) -> &ValueData {
        self.value_opt(id).unwrap()
    }

    fn value_mut(&mut self, id: impl Into<ValueId>) -> &mut ValueData {
        self.value_mut_opt(id).unwrap()
    }

    fn integer_value(&self, id: IntegerId) -> Option<i64> {
        if let ValueData::Integer(i) = self.value_opt(id)? {
            Some(*i)
        } else {
            None
        }
    }

    fn integer_value_mut(&mut self, id: IntegerId) -> Option<&mut i64> {
        if let ValueData::Integer(i) = self.value_mut_opt(id)? {
            Some(i)
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

    fn float_value_mut(&mut self, id: FloatId) -> Option<&mut f64> {
        if let ValueData::Float(f) = self.value_mut_opt(id)? {
            Some(f)
        } else {
            None
        }
    }

    fn str_value(&self, id: StringId) -> Option<&str> {
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

    fn boolean_value_mut(&mut self, id: BooleanId) -> Option<&mut bool> {
        if let ValueData::Boolean(b) = self.value_mut_opt(id)? {
            Some(b)
        } else {
            None
        }
    }

    fn str_value_mut(&mut self, id: StringId) -> Option<&mut str> {
        if let ValueData::Str(s) = self.value_mut_opt(id)? {
            Some(s)
        } else {
            None
        }
    }
}

impl<T> ValueStore for &mut T
where
    T: ValueStore,
{
    fn store<U, V>(&mut self, data: U) -> V
    where
        U: Into<ValueData>,
        V: From<ValueId>,
    {
        (*self).store(data)
    }

    fn value_opt(&self, id: impl Into<ValueId>) -> Option<&ValueData> {
        (**self).value_opt(id)
    }

    fn value_mut_opt(&mut self, id: impl Into<ValueId>) -> Option<&mut ValueData> {
        (*self).value_mut_opt(id)
    }
}

impl From<i64> for ValueData {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for ValueData {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for ValueData {
    fn from(v: String) -> Self {
        Self::Str(v)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultValueStore(Vec<ValueData>);

impl ValueStore for DefaultValueStore {
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

    fn value_opt(&self, id: impl Into<ValueId>) -> Option<&ValueData> {
        self.0.get(id.into().0 as usize)
    }

    fn value_mut_opt(&mut self, id: impl Into<ValueId>) -> Option<&mut ValueData> {
        self.0.get_mut(id.into().0 as usize)
    }
}
