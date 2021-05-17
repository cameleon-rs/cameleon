//! This module contains types which implement either one of `IInterface` defined in `GenICam
//! Starndard`.

use cameleon_genapi::{
    elem_type::{DisplayNotation, FloatRepresentation, IntegerRepresentation},
    interface::IncrementMode,
    prelude::*,
    Device, EnumEntryNode, GenApiResult, NodeId,
};

use super::{GenApiCtxt, ParamsCtxt};

/// A node that has `IInteger` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerNode(NodeId);

/// A node that has `IFloat` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloatNode(NodeId);

/// A node that has `IString` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringNode(NodeId);

/// A node that has `IEnumeration` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumerationNode(NodeId);

/// A node that has `ICommand` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandNode(NodeId);

/// A node that has `IBoolean` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BooleanNode(NodeId);

/// A node that has `IRegister` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterNode(NodeId);

/// A node that has `Category` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryNode(NodeId);

/// A node that has `IPort` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortNode(NodeId);

/// An uninterpreted node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(pub(super) NodeId);

impl From<NodeId> for Node {
    fn from(nid: NodeId) -> Self {
        Node(nid)
    }
}

macro_rules! delegate {
    (
        $expect_kind:ident,
        $(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident<$Ctrl:ident, $Ctxt:ident>($self:ident, ctxt: &mut ParamsCtxt<Ctrl, Ctxt> $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty,)*) => {
        $(
            $(#[$meta])*
            $vis fn $method<$Ctrl, $Ctxt>($self, ctxt: &mut ParamsCtxt<$Ctrl, $Ctxt> $(,$arg: $arg_ty)*) -> $ret_ty
            where $Ctrl: Device,
                  $Ctxt: GenApiCtxt
            {
                ctxt.enter2(|ctrl, ns, vc| {
                    $self.0
                        .$expect_kind(ns)
                        .unwrap()
                        .$method($($arg,)* ctrl, ns, vc)
                })
            }
        )*
    };

    (
        no_vc,
        $expect_kind:ident,
        $(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident<$Ctrl:ident, $Ctxt:ident>($self:ident, ctxt: &ParamsCtxt<Ctrl, Ctxt> $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty,)*) => {
        $(
            $(#[$meta])*
            $vis fn $method<$Ctrl, $Ctxt>($self, ctxt: &ParamsCtxt<$Ctrl, $Ctxt> $(,$arg: $arg_ty)*) -> $ret_ty
            where
                  $Ctxt: GenApiCtxt
            {
                let ns = ctxt.node_store();
                $self.0.$expect_kind(ns).unwrap().$method($($arg,)* ns)
            }
        )*
    };
}

impl IntegerNode {
    delegate! {
        expect_iinteger_kind,
        /// Returns the value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Sets the value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns the minimum value which the node can take.
        pub fn min<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Restricts minimum value of the node.
        pub fn set_min<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns the maximum value which the node can take.
        pub fn max<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Restricts maximum value of the node.
        pub fn set_max<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns the increment value if `inc_mode` returns IncrementMode::FixedIncrement. The value
        /// to set must be `min + i * Increment`.
        ///
        /// NOTE: Some nodes like `MaskedIntReg` doesn't have this element, though `IInteger`
        /// defines getter of the value.
        pub fn inc<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<Option<i64>>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }
    delegate! {
       no_vc,
       expect_iinteger_kind,
       /// Returns [`IncrementMode`] of the node.
       pub fn inc_mode<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<IncrementMode>,
       /// Returns [`IntegerRepresentation`] of the node. This feature is mainly for GUI.
       pub fn representation<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> IntegerRepresentation,
    }

    /// Returns unit that describes phisical meaning of the value. e.g. "Hz" or "ms".
    pub fn unit<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<String>
    where
        Ctxt: GenApiCtxt,
    {
        let ns = ctxt.node_store();
        self.0
            .expect_iinteger_kind(ns)
            .unwrap()
            .unit(ns)
            .map(String::from)
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl FloatNode {
    delegate! {
        expect_ifloat_kind,
        /// Returns the value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<f64>,
        /// Sets the value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: f64) -> GenApiResult<()>,
        /// Returns minimum value which the node can take.
        pub fn min<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<f64>,
        /// Returns maximum value which the node can take.
        pub fn max<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<f64>,
        /// Returns the increment value if `inc_mode` returns IncrementMode::FixedIncrement. The value
        /// to set must be `min + i * Increment`.
        ///
        /// NOTE: Some nodes like `MaskedIntReg` doesn't have this element, though `IFloat`
        /// defines getter of the value.
        pub fn inc<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<Option<f64>>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }

    delegate! {
       no_vc,
       expect_ifloat_kind,
       /// Returns [`IncrementMode`] of the node.
       pub fn inc_mode<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<IncrementMode>,
       /// Returns [`FloatRepresentation`] of the node. This feature is mainly for GUI.
       pub fn representation<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) ->FloatRepresentation,
       /// Returns [`DisplayNotation`]. This featres is mainly for GUI.
       pub fn display_notation<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> DisplayNotation,
    }

    /// Returns unit that describes phisical meaning of the value. e.g. "Hz" or "ms".
    pub fn unit<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<String>
    where
        Ctxt: GenApiCtxt,
    {
        let ns = ctxt.node_store();
        self.0
            .expect_ifloat_kind(ns)
            .unwrap()
            .unit(ns)
            .map(String::from)
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl StringNode {
    delegate! {
        expect_istring_kind,
        /// Returns the value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<String>,
        /// Sets the value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: String) -> GenApiResult<()>,
        /// Returns the maximum length of the string.
        pub fn max_length<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl EnumerationNode {
    delegate! {
    expect_ienumeration_kind,
        /// Sets entry to the enumeration node by the entry name.
        pub fn set_entry_by_name<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, name: &str) -> GenApiResult<()>,
        /// Sets entry to the enumeration node by the entry value.
        pub fn set_entry_by_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }

    /// Allows the access to entries of the node.
    pub fn with_entries<Ctrl, Ctxt, F, R>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>, f: F) -> R
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
        F: FnOnce(&[EnumEntryNode]) -> R,
    {
        let ns = ctxt.node_store();
        f(self.0.expect_ienumeration_kind(ns).unwrap().entries(ns))
    }

    /// Returns entries of the node.
    ///
    /// This method isn't cheap, consider using [`Self::with_entries`] instead of calling this
    /// method.
    pub fn entries<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Vec<EnumEntryNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        let ns = ctxt.node_store();
        self.0
            .expect_ienumeration_kind(ns)
            .unwrap()
            .entries(ns)
            .to_vec()
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl CommandNode {
    delegate! {
        expect_icommand_kind,
        /// Execute the command.
        pub fn execute<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<()>,
        /// Returns `true` if the previous command is executed on the device.
        pub fn is_done<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable (executable).
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl BooleanNode {
    delegate! {
        expect_iboolean_kind,
        /// Returns the value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Sets the value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: bool) -> GenApiResult<()>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl RegisterNode {
    delegate! {
        expect_iregister_kind,
        /// Reads bytes from the register.
        /// `buf.len()` must be same as the register length returned from [`Self::length`].
        pub fn read<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, buf: &mut [u8]) -> GenApiResult<()>,
        /// Writes bytes to the register.
        ///
        /// `data.len()` must be same as the register length returned from [`IRegister::length`].
        pub fn write<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, data: &[u8]) -> GenApiResult<()>,
        /// Returns the address of the register that the node pointing to.
        pub fn address<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Returns the length of the register that the node pointing to.
        pub fn length<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl CategoryNode {
    /// Returns nodes in the category.
    pub fn nodes<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Vec<Node>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| {
            self.0
                .expect_icategory_kind(ns)
                .unwrap()
                .nodes(ns)
                .iter()
                .map(|nid| Node(*nid))
                .collect()
        })
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

impl PortNode {
    delegate! {
        expect_iport_kind,
        /// Reads bytes.
        pub fn read<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, address: i64, buf: &mut [u8]) -> GenApiResult<()>,
        /// Writes bytes.
        pub fn write<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, address: i64, data: &[u8]) -> GenApiResult<()>,
    }

    /// Upcast to [`Node`].
    pub fn as_node(self) -> Node {
        Node(self.0)
    }
}

macro_rules! downcast {
    ($(
       $(#[$meta:meta])*
       ($method:ident, $expect_kind:ident, $ty:ident),
     )*
    ) => {
        $(
            $(#[$meta])*
            pub fn $method<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<$ty>
            where
                Ctxt: GenApiCtxt,
            {
                let ns = ctxt.node_store();
                self.0.$expect_kind(ns).map(|_| $ty(self.0))
        })*
    };
}

macro_rules! delegate_node_base {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident<$Ctrl:ident, $Ctxt:ident>($self:ident, ctxt: &ParamsCtxt<Ctrl, Ctxt> $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty,)*) => {
        $(
            $(#[$meta])*
            $vis fn $method<$Ctrl, $Ctxt>($self, ctxt: &ParamsCtxt<$Ctrl, $Ctxt> $(,$arg: $arg_ty)*) -> $ret_ty
            where
                  $Ctxt: GenApiCtxt
            {
                let ns = ctxt.node_store();
                let node_base = $self.0.as_inode_kind(ns).unwrap().node_base_precise();
                node_base.$method($($arg,)*).into()
            }
        )*
    };
}

impl Node {
    downcast! {
        /// Try downcasting to [`IntegerNode`]. Returns `None` if downcast failed.
        (as_integer, as_iinteger_kind, IntegerNode),
        /// Try downcasting to [`FloatNode`]. Returns `None` if downcast failed.
        (as_float, as_ifloat_kind, FloatNode),
        /// Try downcasting to [`StringNode`]. Returns `None` if downcast failed.
        (as_string ,as_istring_kind, StringNode),
        /// Try downcasting to [`EnumerationNode`]. Returns `None` if downcast failed.
        (as_enumeration ,as_ienumeration_kind, EnumerationNode),
        /// Try downcasting to [`CommandNode`]. Returns `None` if downcast failed.
        (as_command, as_icommand_kind, CommandNode),
        /// Try downcasting to [`BooleanNode`]. Returns `None` if downcast failed.
        (as_boolean, as_iboolean_kind, BooleanNode),
        /// Try downcasting to [`RegisterNode`]. Returns `None` if downcast failed.
        (as_register, as_iregister_kind, RegisterNode),
        /// Try downcasting to [`CategoryNode`]. Returns `None` if downcast failed.
        (as_category, as_icategory_kind, CategoryNode),
        /// Try downcasting to [`PortNode`]. Returns `None` if downcast failed.
        (as_port, as_iport_kind, PortNode),
    }

    /// Returns name of the node.
    pub fn name<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> &str
    where
        Ctxt: GenApiCtxt,
    {
        let ns = ctxt.node_store();
        self.0.as_inode_kind(ns).unwrap().name(ns)
    }

    /// Returns display name of the node. This method is mainly for GUI.
    pub fn display_name<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> &str
    where
        Ctxt: GenApiCtxt,
    {
        let ns = ctxt.node_store();
        let node_base = self.0.as_inode_kind(ns).unwrap().node_base_precise();

        if let Some(desc) = node_base.display_name() {
            desc
        } else {
            self.name(ctxt)
        }
    }

    delegate_node_base! {
        /// Returns name space of the node.
        pub fn name_space<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> super::NameSpace,
        /// Returns description of the node if exists. This method is mainly for GUI.
        pub fn description<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<&str>,
        /// Returns expose static of the node if exists. This method is mainly for GUI.
        pub fn expose_static<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<bool>,
        /// Returns visibility of the node. This method is mainly for GUI.
        pub fn visibility<Ctrl, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> super::Visibility,
        /// Returns `true` if the node is marked as deprecated.
        pub fn is_deprecated<Ctlr, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> bool,
        /// Returns event id of the node if exists.
        pub fn event_id<Ctlr, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<u64>,
        /// Returns tooltip of the node. This method is mainly for GUI.
        pub fn tooltip<Ctlr, Ctxt>(self, ctxt: &ParamsCtxt<Ctrl, Ctxt>) -> Option<&str>,
    }
}
