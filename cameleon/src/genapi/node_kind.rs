//! This module contains types which implement either one of `IInterface` defined in `GenICam
//! Starndard`.

use cameleon_genapi::{
    elem_type::IntegerRepresentation, interface::IncrementMode, prelude::*, Device, GenApiResult,
    NodeId,
};

use super::{GenApiCtxt, ParamsCtxt};

/// A node that has `IInteger` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Integer(NodeId);

macro_rules! delegate {
    (
        $expect_kind:ident,
        $(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident<$Ctrl:ident, $Ctxt:ident>($self:ident, ctxt: &mut ParamsCtxt<Ctrl, Ctxt> $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty,)*) => {
        $(
            $(#[$meta])*
            $vis fn $method<$Ctrl, $Ctxt>($self $(,$arg: $arg_ty)*, ctxt: &mut ParamsCtxt<$Ctrl, $Ctxt>) -> $ret_ty
            where $Ctrl: Device,
                  $Ctxt: GenApiCtxt
            {
                ctxt.enter(|ctrl, ctxt| {
                    ctxt.enter(|node_store, value_ctxt| {
                        $self.0
                            .$expect_kind(node_store)
                            .unwrap()
                            .$method($($arg,)* ctrl, node_store, value_ctxt)
                    })
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
            $vis fn $method<$Ctrl, $Ctxt>($self $(,$arg: $arg_ty)*, ctxt: &mut ParamsCtxt<$Ctrl, $Ctxt>) -> $ret_ty
            where $Ctrl: Device,
                  $Ctxt: GenApiCtxt
            {
                ctxt.enter(|_, ctxt| {
                    ctxt.enter(|node_store, _| {
                        $self.0
                            .$expect_kind(node_store)
                            .unwrap()
                            .$method($($arg,)*  node_store)
                    })
                })
            }
        )*
    };


}

impl Integer {
    delegate!(
        expect_iinteger_kind,
        /// Returns integer value of the node.
        pub fn value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Sets integer value of the node.
        pub fn set_value<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns minimum value which the node accepts.
        pub fn min<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> GenApiResult<i64>,
        /// Restricts minimum value of the node.
        pub fn set_min<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>, value: i64) -> GenApiResult<()>,
        /// Returns maximum value which the node accepts.
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
    );
    delegate!(
       no_vc,
       expect_iinteger_kind,
       /// Returns [`IncrementMode`] of the node.
       pub fn inc_mode<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> Option<IncrementMode>,
       /// Returns [`IntegerRepresentation`] of the node.
       pub fn representation<Ctrl, Ctxt>(self, ctxt: &mut ParamsCtxt<Ctrl, Ctxt>) -> IntegerRepresentation,
    );
}
