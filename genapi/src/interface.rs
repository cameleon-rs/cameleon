use super::{
    elem_type::{DisplayNotation, FloatRepresentation, IntegerRepresentation},
    store::{CacheStore, NodeData, NodeId, NodeStore, ValueStore},
    EnumEntryNode, {Device, GenApiResult, ValueCtxt},
};

#[derive(Clone, Debug)]
pub enum IncrementMode {
    FixedIncrement,
    /// NOTE: `ListIncrement` is not supported in `GenApiSchema Version 1.1` yet.
    ListIncrement,
}

pub trait IInteger {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>>;

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: impl NodeStore) -> &[i64];

    fn representation(&self, store: impl NodeStore) -> IntegerRepresentation;

    fn unit(&self, store: impl NodeStore) -> Option<&str>;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

pub trait IFloat {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64>;

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>>;

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: impl NodeStore) -> &[f64];

    fn representation(&self, store: impl NodeStore) -> FloatRepresentation;

    fn unit(&self, store: impl NodeStore) -> Option<&str>;

    fn display_notation(&self, store: impl NodeStore) -> DisplayNotation;

    fn display_precision(&self, store: impl NodeStore) -> i64;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

pub trait IString {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;
}

pub trait IEnumeration {
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode>;

    fn entries(&self, store: impl NodeStore) -> GenApiResult<&[EnumEntryNode]>;

    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn set_entry_by_idx<T: ValueStore, U: CacheStore>(
        &self,
        idx: usize,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

pub trait ICommand {
    fn execute<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn is_done<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;
}

pub trait IBoolean {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: bool,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

pub trait IRegister {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64>;
}

pub trait ICategory {
    /// Return nodes in the category.
    fn nodes(&self, store: impl NodeStore) -> GenApiResult<&[NodeId]>;
}

pub trait IPort {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()>;
}

pub trait ISelector {
    /// Return nodes which is selected by the current node.
    fn selected_nodes<T: ValueStore, U: CacheStore>(
        &self,
        store: impl NodeStore,
    ) -> GenApiResult<Vec<NodeId>>;

    /// Return nodes which refer to the current node as a selector.
    fn selecting_nodes<T: ValueStore, U: CacheStore>(
        &self,
        store: impl NodeStore,
    ) -> GenApiResult<Vec<NodeId>>;
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
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.value(device, store, cx))
    }
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_value(value, device, store, cx))
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.min(device, store, cx))
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_iinteger_variant!(self.max(device, store, cx))
    }

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        delegate_to_iinteger_variant!(self.inc_mode(store))
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        delegate_to_iinteger_variant!(self.inc(device, store, cx))
    }

    fn valid_value_set(&self, store: impl NodeStore) -> &[i64] {
        delegate_to_iinteger_variant!(self.valid_value_set(store))
    }

    fn representation(&self, store: impl NodeStore) -> IntegerRepresentation {
        delegate_to_iinteger_variant!(self.representation(store))
    }

    fn unit(&self, store: impl NodeStore) -> Option<&str> {
        delegate_to_iinteger_variant!(self.unit(store))
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_min(value, device, store, cx))
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_iinteger_variant!(self.set_max(value, device, store, cx))
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
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.value(device, store, cx))
    }
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_value(value, device, store, cx))
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.min(device, store, cx))
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        delegate_to_ifloat_variant!(self.max(device, store, cx))
    }

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        delegate_to_ifloat_variant!(self.inc_mode(store))
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        delegate_to_ifloat_variant!(self.inc(device, store, cx))
    }

    fn valid_value_set(&self, store: impl NodeStore) -> &[f64] {
        delegate_to_ifloat_variant!(self.valid_value_set(store))
    }

    fn representation(&self, store: impl NodeStore) -> FloatRepresentation {
        delegate_to_ifloat_variant!(self.representation(store))
    }

    fn unit(&self, store: impl NodeStore) -> Option<&str> {
        delegate_to_ifloat_variant!(self.unit(store))
    }

    fn display_notation(&self, store: impl NodeStore) -> DisplayNotation {
        delegate_to_ifloat_variant!(self.display_notation(store))
    }

    fn display_precision(&self, store: impl NodeStore) -> i64 {
        delegate_to_ifloat_variant!(self.display_precision(store))
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_min(value, device, store, cx))
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_ifloat_variant!(self.set_max(value, device, store, cx))
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
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String> {
        delegate_to_istring_variant!(self.value(device, store, cx))
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        delegate_to_istring_variant!(self.set_value(value, device, store, cx))
    }

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        delegate_to_istring_variant!(self.max_length(device, store, cx))
    }
}
