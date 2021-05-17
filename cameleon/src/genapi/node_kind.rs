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
            $vis:vis fn $method:ident<$Ctrl:ident, $Ctxt:ident>($self:ident, ctxt: &mut ParamsCtxt<Ctrl, Ctxt> $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty,)*) => {
        $(
            $(#[$meta])*
            $vis fn $method<$Ctrl, $Ctxt>($self, ctxt: &ParamsCtxt<$Ctrl, $Ctxt> $(,$arg: $arg_ty)*) -> $ret_ty
            where $Ctrl: Device,
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
       pub fn inc_mode<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<IncrementMode>,
       /// Returns [`IntegerRepresentation`] of the node. This feature is mainly for GUI.
       pub fn representation<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> IntegerRepresentation,
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
       pub fn inc_mode<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<IncrementMode>,
       /// Returns [`FloatRepresentation`] of the node. This feature is mainly for GUI.
       pub fn representation<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) ->FloatRepresentation,
       /// Returns [`DisplayNotation`]. This featres is mainly for GUI.
       pub fn display_notation<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> DisplayNotation,
    }

    /// Returns unit that describes phisical meaning of the value. e.g. "Hz" or "ms". This
    /// feature is mainly for GUI.
    pub fn unit<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<String>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| {
            self.0
                .expect_ifloat_kind(ns)
                .unwrap()
                .unit(ns)
                .map(String::from)
        })
    }
}

impl StringNode {
    delegate! {
        expect_istring_kind,
        /// Returns the value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<String>,
        /// Sets the value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: String) -> GenApiResult<()>,
        /// Retruns the maximum length of the string.
        pub fn max_length<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Returns `true` if the node is readable.
        pub fn is_readable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
        /// Returns `true` if the node is writable.
        pub fn is_writable<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<bool>,
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
    pub fn with_entries<Ctrl, Ctxt, F, R>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, f: F) -> R
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
        F: FnOnce(&[EnumEntryNode]) -> R,
    {
        ctxt.enter2(|_, ns, _| f(self.0.expect_ienumeration_kind(ns).unwrap().entries(ns)))
    }

    /// Returns entries of the node.
    ///
    /// This method isn't cheap, consider using [`Self::with_entries`] instead of calling this
    /// method.
    pub fn entries<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Vec<EnumEntryNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| {
            self.0
                .expect_ienumeration_kind(ns)
                .unwrap()
                .entries(ns)
                .to_vec()
        })
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
        /// Return the length of the register that the node pointing to.
        pub fn length<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
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
}

impl PortNode {
    delegate! {
        expect_iport_kind,
        /// Reads bytes.
        pub fn read<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, address: i64, buf: &mut [u8]) -> GenApiResult<()>,
        /// Writes bytes.
        pub fn write<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, address: i64, data: &[u8]) -> GenApiResult<()>,
    }
}

impl Node {
    /// Returns `None` if downcast is failed.
    pub fn as_integer<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<IntegerNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_iinteger_kind(ns).map(|_| IntegerNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_float<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<FloatNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_ifloat_kind(ns).map(|_| FloatNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_string<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<StringNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_istring_kind(ns).map(|_| StringNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_ienumeration<Ctrl, Ctxt>(
        self,
        ctxt: &mut ParamsCtxt<Ctrl, Ctxt>,
    ) -> Option<EnumerationNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| {
            self.0
                .as_ienumeration_kind(ns)
                .map(|_| EnumerationNode(self.0))
        })
    }

    /// Returns `None` if downcast is failed.
    pub fn as_command<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<CommandNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_icommand_kind(ns).map(|_| CommandNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_boolean<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<BooleanNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_iboolean_kind(ns).map(|_| BooleanNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_register<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<RegisterNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_iregister_kind(ns).map(|_| RegisterNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_category<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<CategoryNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_icategory_kind(ns).map(|_| CategoryNode(self.0)))
    }

    /// Returns `None` if downcast is failed.
    pub fn as_port<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<PortNode>
    where
        Ctrl: Device,
        Ctxt: GenApiCtxt,
    {
        ctxt.enter2(|_, ns, _| self.0.as_iport_kind(ns).map(|_| PortNode(self.0)))
    }
}
