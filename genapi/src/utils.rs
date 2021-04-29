use std::{borrow::Cow, collections::HashMap, convert::TryInto};

use super::{
    elem_type::{Endianness, NamedValue, Sign},
    formula::Expr,
    interface::{IBoolean, IEnumeration, IFloat, IInteger},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

pub(super) fn bool_from_id<T: ValueStore, U: CacheStore>(
    node_id: NodeId,
    device: &mut impl Device,
    store: &impl NodeStore,
    cx: &mut ValueCtxt<T, U>,
) -> GenApiResult<bool> {
    if let Some(node) = node_id.as_iboolean_kind(store) {
        node.value(device, store, cx)
    } else if let Some(node) = node_id.as_iinteger_kind(store) {
        Ok(node.value(device, store, cx)? == 1)
    } else {
        Err(GenApiError::InvalidNode(
            "the node doesn't implement `IInteger` nor `IBoolean".into(),
        ))
    }
}

pub(super) fn int_from_slice(
    slice: &[u8],
    endianness: Endianness,
    sign: Sign,
) -> GenApiResult<i64> {
    macro_rules! convert_from_slice {
        ($(($len:literal, $signed_ty:ty, $unsigned_ty:ty)),*) => {
            match (slice.len(), endianness, sign) {
                $(
                    ($len, Endianness::LE, Sign::Signed) => Ok(<$signed_ty>::from_le_bytes(slice.try_into().unwrap()) as i64),
                    ($len, Endianness::LE, Sign::Unsigned) => Ok(<$unsigned_ty>::from_be_bytes(slice.try_into().unwrap()) as i64),
                    ($len, Endianness::BE, Sign::Signed) => Ok(<$signed_ty>::from_be_bytes(slice.try_into().unwrap()) as i64),
                    ($len, Endianness::BE, Sign::Unsigned) => Ok(<$unsigned_ty>::from_be_bytes(slice.try_into().unwrap()) as i64),
                )*
                _ => Err(GenApiError::InvalidBuffer("buffer lenght must be either 1/2/4/8 to convert to i64".into()))
            }
        }
    }

    convert_from_slice!((8, i64, u64), (4, i32, u32), (2, i16, u16), (1, i8, u8))
}

pub(super) fn bytes_from_int(
    value: i64,
    buf: &mut [u8],
    endianness: Endianness,
    sign: Sign,
) -> GenApiResult<()> {
    macro_rules! convert_to_slice {
        ($(($len:literal, $signed_ty:ty, $unsigned_ty:ty)),*) => {
            match (buf.len(), endianness, sign) {
                $(
                    ($len, Endianness::LE, Sign::Signed) => Ok(buf.copy_from_slice(&(value as $signed_ty).to_le_bytes())),
                    ($len, Endianness::LE, Sign::Unsigned) => Ok(buf.copy_from_slice(&(value as $unsigned_ty).to_le_bytes())),
                    ($len, Endianness::BE, Sign::Signed) => Ok(buf.copy_from_slice(&(value as $signed_ty).to_be_bytes())),
                    ($len, Endianness::BE, Sign::Unsigned) => Ok(buf.copy_from_slice(&(value as $unsigned_ty).to_be_bytes())),
                )*
                _ => Err(GenApiError::InvalidBuffer("buffer lenght must be either 1/2/4/8 to convert to i64".into()))
            }
        }
    }

    convert_to_slice!((8, i64, u64), (4, i32, u32), (2, i16, u16), (1, i8, u8))
}

pub(super) fn verify_value_in_range<T>(value: T, min: T, max: T) -> GenApiResult<()>
where
    T: PartialOrd,
{
    if value < min {
        Err(GenApiError::InvalidData(
            "given data is smaller than min value of the node".into(),
        ))
    } else if value > max {
        Err(GenApiError::InvalidData(
            "given data is larger than max value of the node".into(),
        ))
    } else {
        Ok(())
    }
}

pub(super) struct FormulaEnvCollector<'a, T> {
    p_variables: &'a [NamedValue<NodeId>],
    constants: &'a [NamedValue<T>],
    expressions: &'a [NamedValue<Expr>],
}

impl<'a, T: Copy + Into<Expr>> FormulaEnvCollector<'a, T> {
    pub(super) fn collect<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<HashMap<&'a str, Cow<'a, Expr>>> {
        // Collect variables.
        let mut var_env = self.collect_variables(device, store, cx)?;

        // Collect constatns.
        for constant in self.constants {
            let name = constant.name();
            let value: Expr = (constant.value()).into();
            var_env.insert(name, Cow::Owned(value));
        }

        // Collect expressions.
        for expr in self.expressions {
            let name = expr.name();
            let value = expr.value_ref();
            var_env.insert(name, Cow::Borrowed(value));
        }

        Ok(var_env)
    }

    fn collect_variables<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<HashMap<&'a str, Cow<'a, Expr>>> {
        let mut var_env = HashMap::new();

        for variable in self.p_variables {
            let name = variable.name();
            let nid = variable.value();
            let expr = VariableKind::from_str(name).get_value(nid, device, store, cx)?;
            var_env.insert(name, Cow::Owned(expr));
        }
        Ok(var_env)
    }
}

enum VariableKind<'a> {
    Value,
    Min,
    Max,
    Inc,
    Enum(&'a str),
}

impl<'a> VariableKind<'a> {
    fn from_str(s: &'a str) -> Self {
        let split: Vec<&'a str> = s.splitn(3, '.').collect();
        match split.as_slice() {
            [_] | [_, "Value"] => Self::Value,
            [_, "Min"] => Self::Min,
            [_, "Max"] => Self::Max,
            [_, "Inc"] => Self::Inc,
            [_, "Enum", name] => Self::Enum(name),
            _ => panic!("invalid `pVariable`: {}", s),
        }
    }

    fn get_value<T: ValueStore, U: CacheStore>(
        self,
        nid: NodeId,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Expr> {
        fn error(nid: NodeId, store: &impl NodeStore) -> GenApiError {
            GenApiError::InvalidNode(format!("invalid `pVariable: {}`", nid.name(store)).into())
        }

        let expr: Expr = match self {
            Self::Value => {
                if let Some(node) = nid.as_iinteger_kind(store) {
                    node.value(device, store, cx)?.into()
                } else if let Some(node) = nid.as_ifloat_kind(store) {
                    node.value(device, store, cx)?.into()
                } else if let Some(node) = nid.as_iboolean_kind(store) {
                    node.value(device, store, cx)?.into()
                } else {
                    return Err(error(nid, store));
                }
            }
            Self::Min => {
                if let Some(node) = nid.as_iinteger_kind(store) {
                    node.min(device, store, cx)?.into()
                } else if let Some(node) = nid.as_ifloat_kind(store) {
                    node.min(device, store, cx)?.into()
                } else {
                    return Err(error(nid, store));
                }
            }
            Self::Max => {
                if let Some(node) = nid.as_iinteger_kind(store) {
                    node.max(device, store, cx)?.into()
                } else if let Some(node) = nid.as_ifloat_kind(store) {
                    node.max(device, store, cx)?.into()
                } else {
                    return Err(error(nid, store));
                }
            }
            Self::Inc => {
                if let Some(node) = nid.as_iinteger_kind(store) {
                    node.inc(device, store, cx)?
                        .ok_or_else(|| error(nid, store))?
                        .into()
                } else if let Some(node) = nid.as_ifloat_kind(store) {
                    node.inc(device, store, cx)?
                        .ok_or_else(|| error(nid, store))?
                        .into()
                } else {
                    return Err(error(nid, store));
                }
            }
            Self::Enum(name) => {
                if let Some(node) = nid.as_ienumeration_kind(store) {
                    node.entry_by_name(name, store)?
                        .ok_or_else(|| error(nid, store))?
                        .value()
                        .into()
                } else {
                    return Err(error(nid, store));
                }
            }
        };

        Ok(expr)
    }
}
