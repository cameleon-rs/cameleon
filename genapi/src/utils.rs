use std::convert::TryInto;

use super::{
    elem_type::{Endianness, Sign},
    interface::{IBoolean, IInteger},
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
