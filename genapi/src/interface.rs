/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ambassador::{delegatable_trait, Delegate};

use super::{
    elem_type::{DisplayNotation, FloatRepresentation, IntegerRepresentation},
    node_base::NodeBase,
    store::{CacheStore, NodeData, NodeId, NodeStore, ValueStore},
    {Device, GenApiResult, ValueCtxt},
};

#[derive(Clone, Debug)]
pub enum IncrementMode {
    FixedIncrement,
    /// NOTE: `ListIncrement` is not supported in `GenApiSchema Version 1.1` yet.
    ListIncrement,
}

#[delegatable_trait]
pub trait INode {
    fn name<'s>(&self, store: &'s impl NodeStore) -> &'s str {
        store.name_by_id(self.node_base().id()).unwrap()
    }

    fn node_base(&self) -> NodeBase<'_>;
    fn streamable(&self) -> bool;
}

#[delegatable_trait]
pub trait IInteger {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn inc_mode(&self, store: &impl NodeStore) -> Option<IncrementMode>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>>;

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: &impl NodeStore) -> &[i64];

    fn representation(&self, store: &impl NodeStore) -> IntegerRepresentation;

    fn unit(&self, store: &impl NodeStore) -> Option<&str>;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait IFloat {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn inc_mode(&self, store: &impl NodeStore) -> Option<IncrementMode>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>>;

    fn representation(&self, store: &impl NodeStore) -> FloatRepresentation;

    fn unit(&self, store: &impl NodeStore) -> Option<&str>;

    fn display_notation(&self, store: &impl NodeStore) -> DisplayNotation;

    fn display_precision(&self, store: &impl NodeStore) -> i64;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait IString {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: String,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait IEnumeration {
    fn current_value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<NodeId>;

    fn entries(&self, store: &impl NodeStore) -> &[NodeId];

    /// Get [`NodeId`] of enum entry which has specified symbolic name.
    fn entry_by_symbolic(&self, name: &str, store: &impl NodeStore) -> Option<NodeId> {
        for nid in self.entries(store) {
            let ent = nid.expect_enum_entry(store).unwrap(); // Never fail when parse is succeeded.
            if ent.symbolic() == name {
                return Some(*nid);
            }
        }
        None
    }

    fn set_entry_by_symbolic<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_entry_by_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait ICommand {
    fn execute<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_done<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait IBoolean {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: bool,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

#[delegatable_trait]
pub trait IRegister {
    /// Read bytes from the register.
    ///
    /// `buf.len()` must be same as the register length returned from [`IRegister::length`].
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    /// Write bytes to the register.
    ///
    /// `buf.len()` must be same as the register length returned from [`IRegister::length`].
    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;
}

#[delegatable_trait]
pub trait ICategory {
    /// Return nodes in the category.
    fn nodes(&self, store: &impl NodeStore) -> &[NodeId];
}

#[delegatable_trait]
pub trait IPort {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

#[delegatable_trait]
pub trait ISelector {
    /// Return nodes which refer to the current node as a selector.
    fn selecting_nodes(&self, store: &impl NodeStore) -> GenApiResult<&[NodeId]>;
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(INode)]
pub enum INodeKind<'a> {
    Integer(&'a super::IntegerNode),
    IntReg(&'a super::IntRegNode),
    MaskedIntReg(&'a super::MaskedIntRegNode),
    IntConverter(&'a super::IntConverterNode),
    IntSwissKnife(&'a super::IntSwissKnifeNode),
    Float(&'a super::FloatNode),
    FloatReg(&'a super::FloatRegNode),
    Converter(&'a super::ConverterNode),
    SwissKnife(&'a super::SwissKnifeNode),
    String(&'a super::StringNode),
    StringReg(&'a super::StringRegNode),
    Boolean(&'a super::BooleanNode),
    Command(&'a super::CommandNode),
    Register(&'a super::RegisterNode),
    Category(&'a super::CategoryNode),
    Port(&'a super::PortNode),
    Enumeration(&'a super::EnumerationNode),
    EnumEntry(&'a super::EnumEntryNode),
    Node(&'a super::Node),
}

impl<'a> INodeKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Integer(n) => Some(Self::Integer(n)),
            NodeData::IntReg(n) => Some(Self::IntReg(n)),
            NodeData::MaskedIntReg(n) => Some(Self::MaskedIntReg(n)),
            NodeData::IntConverter(n) => Some(Self::IntConverter(n)),
            NodeData::IntSwissKnife(n) => Some(Self::IntSwissKnife(n)),
            NodeData::Float(n) => Some(Self::Float(n)),
            NodeData::FloatReg(n) => Some(Self::FloatReg(n)),
            NodeData::Converter(n) => Some(Self::Converter(n)),
            NodeData::SwissKnife(n) => Some(Self::SwissKnife(n)),
            NodeData::String(n) => Some(Self::String(n)),
            NodeData::StringReg(n) => Some(Self::StringReg(n)),
            NodeData::Boolean(n) => Some(Self::Boolean(n)),
            NodeData::Command(n) => Some(Self::Command(n)),
            NodeData::Register(n) => Some(Self::Register(n)),
            NodeData::Category(n) => Some(Self::Category(n)),
            NodeData::Port(n) => Some(Self::Port(n)),
            NodeData::Enumeration(n) => Some(Self::Enumeration(n)),
            NodeData::EnumEntry(n) => Some(Self::EnumEntry(n)),
            NodeData::Node(n) => Some(Self::Node(n)),
            _ => None,
        }
    }

    /// Returns [`NodeBase`] with more precise lifetime.
    pub fn node_base_precise(self) -> NodeBase<'a> {
        match self {
            Self::Integer(n) => n.node_base(),
            Self::IntReg(n) => n.node_base(),
            Self::MaskedIntReg(n) => n.node_base(),
            Self::IntConverter(n) => n.node_base(),
            Self::IntSwissKnife(n) => n.node_base(),
            Self::Float(n) => n.node_base(),
            Self::FloatReg(n) => n.node_base(),
            Self::Converter(n) => n.node_base(),
            Self::SwissKnife(n) => n.node_base(),
            Self::String(n) => n.node_base(),
            Self::StringReg(n) => n.node_base(),
            Self::Boolean(n) => n.node_base(),
            Self::Command(n) => n.node_base(),
            Self::Register(n) => n.node_base(),
            Self::Category(n) => n.node_base(),
            Self::Port(n) => n.node_base(),
            Self::Enumeration(n) => n.node_base(),
            Self::EnumEntry(n) => n.node_base(),
            Self::Node(n) => n.node_base(),
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IInteger)]
pub enum IIntegerKind<'a> {
    Integer(&'a super::IntegerNode),
    IntReg(&'a super::IntRegNode),
    MaskedIntReg(&'a super::MaskedIntRegNode),
    IntConverter(&'a super::IntConverterNode),
    IntSwissKnife(&'a super::IntSwissKnifeNode),
}

impl<'a> IIntegerKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Integer(n) => Some(Self::Integer(n)),
            NodeData::IntReg(n) => Some(Self::IntReg(n)),
            NodeData::MaskedIntReg(n) => Some(Self::MaskedIntReg(n)),
            NodeData::IntConverter(n) => Some(Self::IntConverter(n)),
            NodeData::IntSwissKnife(n) => Some(Self::IntSwissKnife(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IFloat)]
pub enum IFloatKind<'a> {
    Float(&'a super::FloatNode),
    FloatReg(&'a super::FloatRegNode),
    Converter(&'a super::ConverterNode),
    SwissKnife(&'a super::SwissKnifeNode),
}

impl<'a> IFloatKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Float(n) => Some(Self::Float(n)),
            NodeData::FloatReg(n) => Some(Self::FloatReg(n)),
            NodeData::Converter(n) => Some(Self::Converter(n)),
            NodeData::SwissKnife(n) => Some(Self::SwissKnife(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IString)]
pub enum IStringKind<'a> {
    String(&'a super::StringNode),
    StringReg(&'a super::StringRegNode),
}

impl<'a> IStringKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::String(n) => Some(Self::String(n)),
            NodeData::StringReg(n) => Some(Self::StringReg(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(ICommand)]
pub enum ICommandKind<'a> {
    Command(&'a super::CommandNode),
}

impl<'a> ICommandKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Command(n) => Some(Self::Command(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IEnumeration)]
pub enum IEnumerationKind<'a> {
    Enumeration(&'a super::EnumerationNode),
}

impl<'a> IEnumerationKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Enumeration(n) => Some(Self::Enumeration(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IBoolean)]
pub enum IBooleanKind<'a> {
    Boolean(&'a super::BooleanNode),
}

impl<'a> IBooleanKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Boolean(n) => Some(Self::Boolean(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IRegister)]
pub enum IRegisterKind<'a> {
    Register(&'a super::RegisterNode),
    IntReg(&'a super::IntRegNode),
    MaskedIntReg(&'a super::MaskedIntRegNode),
    StringReg(&'a super::StringRegNode),
    FloatReg(&'a super::FloatRegNode),
}

impl<'a> IRegisterKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Register(n) => Some(Self::Register(n)),
            NodeData::IntReg(n) => Some(Self::IntReg(n)),
            NodeData::MaskedIntReg(n) => Some(Self::MaskedIntReg(n)),
            NodeData::StringReg(n) => Some(Self::StringReg(n)),
            NodeData::FloatReg(n) => Some(Self::FloatReg(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(ICategory)]
pub enum ICategoryKind<'a> {
    Category(&'a super::CategoryNode),
}

impl<'a> ICategoryKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Category(n) => Some(Self::Category(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(IPort)]
pub enum IPortKind<'a> {
    Port(&'a super::PortNode),
}

impl<'a> IPortKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Port(n) => Some(Self::Port(n)),
            _ => None,
        }
    }
}

#[derive(Delegate, Clone, Copy, Debug)]
#[delegate(ISelector)]
pub enum ISelectorKind<'a> {
    Integer(&'a super::IntegerNode),
    IntReg(&'a super::IntRegNode),
    MaskedIntReg(&'a super::MaskedIntRegNode),
    Boolean(&'a super::BooleanNode),
    Enumeration(&'a super::EnumerationNode),
}

impl<'a> ISelectorKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Integer(n) => Some(Self::Integer(n)),
            NodeData::IntReg(n) => Some(Self::IntReg(n)),
            NodeData::MaskedIntReg(n) => Some(Self::MaskedIntReg(n)),
            NodeData::Boolean(n) => Some(Self::Boolean(n)),
            NodeData::Enumeration(n) => Some(Self::Enumeration(n)),
            _ => None,
        }
    }
}
