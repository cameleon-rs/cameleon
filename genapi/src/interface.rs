use super::{
    elem_type::{
        DisplayNotation, FloatRepresentation, IntegerRepresentation, NameSpace, Visibility,
    },
    store::{CacheStore, NodeData, NodeId, NodeStore, ValueStore},
    EnumEntryNode, {Device, GenApiResult, ValueCtxt},
};

#[derive(Clone, Debug)]
pub enum IncrementMode {
    FixedIncrement,
    /// NOTE: `ListIncrement` is not supported in `GenApiSchema Version 1.1` yet.
    ListIncrement,
}

pub trait INode {
    fn name_space(&self, store: &impl NodeStore) -> NameSpace;

    fn tool_tip(&self, store: &impl NodeStore) -> Option<&str>;
    fn description(&self, store: &impl NodeStore) -> Option<&str>;
    fn display_name(&self, store: &impl NodeStore) -> &str;
    fn visibility(&self) -> Visibility;
    fn event_id(&self) -> Option<u64>;

    fn is_deprecated<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> bool;
    fn is_available<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> bool;
    fn is_locked<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> bool;
    fn is_implemented<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> bool;
}

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

pub trait IString {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
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

pub trait IEnumeration {
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode>;

    fn entries(&self, store: &impl NodeStore) -> &[EnumEntryNode];

    fn entry_by_name(&self, name: &str, store: &impl NodeStore) -> Option<&EnumEntryNode> {
        self.entries(store)
            .iter()
            .find(|ent| ent.node_base().id().name(store) == name)
    }

    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
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

pub trait ICategory {
    /// Return nodes in the category.
    fn nodes(&self, store: &impl NodeStore) -> &[NodeId];
}

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

pub trait ISelector {
    /// Return nodes which refer to the current node as a selector.
    fn selecting_nodes(&self, store: &impl NodeStore) -> GenApiResult<&[NodeId]>;
}

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

macro_rules! delegate_to_iinteger_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IIntegerKind::Integer(n) => n.$method($($arg),*),
            IIntegerKind::IntReg(n) => n.$method($($arg),*),
            IIntegerKind::MaskedIntReg(n) => n.$method($($arg),*),
            IIntegerKind::IntConverter(n) => n.$method($($arg),*),
            IIntegerKind::IntSwissKnife(n) => n.$method($($arg),*),
        }
    }
}

impl<'a> IInteger for IIntegerKind<'a> {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.value(device, store, cx))
    }
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_value(value, device, store, cx))
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.min(device, store, cx))
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.max(device, store, cx))
    }

    fn inc_mode(&self, store: &impl NodeStore) -> Option<IncrementMode> {
        delegate_to_iinteger_variant!(self.inc_mode(store))
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        delegate_to_iinteger_variant!(self.inc(device, store, cx))
    }

    fn valid_value_set(&self, store: &impl NodeStore) -> &[i64] {
        delegate_to_iinteger_variant!(self.valid_value_set(store))
    }

    fn representation(&self, store: &impl NodeStore) -> IntegerRepresentation {
        delegate_to_iinteger_variant!(self.representation(store))
    }

    fn unit(&self, store: &impl NodeStore) -> Option<&str> {
        delegate_to_iinteger_variant!(self.unit(store))
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_min(value, device, store, cx))
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_max(value, device, store, cx))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_iinteger_variant!(self.is_readable(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_iinteger_variant!(self.is_readable(device, store, cx))
    }
}

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

macro_rules! delegate_to_ifloat_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IFloatKind::Float(n) => n.$method($($arg),*),
            IFloatKind::FloatReg(n) => n.$method($($arg),*),
            IFloatKind::Converter(n) => n.$method($($arg),*),
            IFloatKind::SwissKnife(n) => n.$method($($arg),*),
        }
    }
}

impl<'a> IFloat for IFloatKind<'a> {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.value(device, store, cx))
    }
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_value(value, device, store, cx))
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.min(device, store, cx))
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.max(device, store, cx))
    }

    fn inc_mode(&self, store: &impl NodeStore) -> Option<IncrementMode> {
        delegate_to_ifloat_variant!(self.inc_mode(store))
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        delegate_to_ifloat_variant!(self.inc(device, store, cx))
    }

    fn representation(&self, store: &impl NodeStore) -> FloatRepresentation {
        delegate_to_ifloat_variant!(self.representation(store))
    }

    fn unit(&self, store: &impl NodeStore) -> Option<&str> {
        delegate_to_ifloat_variant!(self.unit(store))
    }

    fn display_notation(&self, store: &impl NodeStore) -> DisplayNotation {
        delegate_to_ifloat_variant!(self.display_notation(store))
    }

    fn display_precision(&self, store: &impl NodeStore) -> i64 {
        delegate_to_ifloat_variant!(self.display_precision(store))
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_min(value, device, store, cx))
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_max(value, device, store, cx))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_ifloat_variant!(self.is_readable(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_ifloat_variant!(self.is_readable(device, store, cx))
    }
}

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

macro_rules! delegate_to_istring_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IStringKind::String(n) => n.$method($($arg),*),
            IStringKind::StringReg(n) => n.$method($($arg),*),
        }
    }
}

impl<'a> IString for IStringKind<'a> {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String> {
        delegate_to_istring_variant!(self.value(device, store, cx))
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_istring_variant!(self.set_value(value, device, store, cx))
    }

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_istring_variant!(self.max_length(device, store, cx))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_istring_variant!(self.is_readable(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_istring_variant!(self.is_writable(device, store, cx))
    }
}

pub enum ICommandKind<'a> {
    Command(&'a super::CommandNode),
}

macro_rules! delegate_to_icommand_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            ICommandKind::Command(n) => n.$method($($arg),*)
        }
    }
}

impl<'a> ICommandKind<'a> {
    pub(super) fn maybe_from(id: NodeId, store: &'a impl NodeStore) -> Option<Self> {
        match store.node_opt(id)? {
            NodeData::Command(n) => Some(Self::Command(n)),
            _ => None,
        }
    }
}

impl<'a> ICommand for ICommandKind<'a> {
    fn execute<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_icommand_variant!(self.execute(device, store, cx))
    }

    fn is_done<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_icommand_variant!(self.is_done(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_icommand_variant!(self.is_writable(device, store, cx))
    }
}

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

macro_rules! delegate_to_ienumeration_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IEnumerationKind::Enumeration(n) => n.$method($($arg),*)
        }
    }
}

impl<'a> IEnumeration for IEnumerationKind<'a> {
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode> {
        delegate_to_ienumeration_variant!(self.current_entry(device, store, cx))
    }

    fn entries(&self, store: &impl NodeStore) -> &[EnumEntryNode] {
        delegate_to_ienumeration_variant!(self.entries(store))
    }

    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ienumeration_variant!(self.set_entry_by_name(name, device, store, cx))
    }

    fn set_entry_by_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ienumeration_variant!(self.set_entry_by_value(value, device, store, cx))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_ienumeration_variant!(self.is_readable(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_ienumeration_variant!(self.is_writable(device, store, cx))
    }
}

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

macro_rules! delegate_to_iboolean_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IBooleanKind::Boolean(n) => n.$method($($arg),*)
        }
    }
}

impl<'a> IBoolean for IBooleanKind<'a> {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_iboolean_variant!(self.value(device, store, cx))
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: bool,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iboolean_variant!(self.set_value(value, device, store, cx))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_iboolean_variant!(self.is_readable(device, store, cx))
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        delegate_to_iboolean_variant!(self.is_writable(device, store, cx))
    }
}

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

macro_rules! delegate_to_iregister_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IRegisterKind::Register(n) => n.$method($($arg),*),
            IRegisterKind::IntReg(n) => n.$method($($arg),*),
            IRegisterKind::MaskedIntReg(n) => n.$method($($arg),*),
            IRegisterKind::StringReg(n) => n.$method($($arg),*),
            IRegisterKind::FloatReg(n) => n.$method($($arg),*),
        }
    }
}

impl<'a> IRegister for IRegisterKind<'a> {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iregister_variant!(self.read(buf, device, store, cx))
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iregister_variant!(self.write(buf, device, store, cx))
    }

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iregister_variant!(self.address(device, store, cx))
    }

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iregister_variant!(self.length(device, store, cx))
    }
}

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

macro_rules! delegate_to_icategory_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            ICategoryKind::Category(n) => n.$method($($arg),*)
        }
    }
}

impl<'a> ICategory for ICategoryKind<'a> {
    fn nodes(&self, store: &impl NodeStore) -> &[NodeId] {
        delegate_to_icategory_variant!(self.nodes(store))
    }
}

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

macro_rules! delegate_to_iport_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            IPortKind::Port(n) => n.$method($($arg),*)
        }
    }
}

impl<'a> IPort for IPortKind<'a> {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iport_variant!(self.read(address, buf, device, store, cx))
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iport_variant!(self.write(address, buf, device, store, cx))
    }
}

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

macro_rules! delegate_to_iselector_variant {
    ($self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            ISelectorKind::Integer(n) => n.$method($($arg),*),
            ISelectorKind::IntReg(n) => n.$method($($arg),*),
            ISelectorKind::MaskedIntReg(n) => n.$method($($arg),*),
            ISelectorKind::Boolean(n) => n.$method($($arg),*),
            ISelectorKind::Enumeration(n) => n.$method($($arg),*),
        }
    }
}

impl<'a> ISelector for ISelectorKind<'a> {
    fn selecting_nodes(&self, store: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        delegate_to_iselector_variant!(self.selecting_nodes(store))
    }
}
