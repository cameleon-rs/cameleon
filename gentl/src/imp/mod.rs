pub(super) mod device;
pub(super) mod interface;
pub(super) mod port;
pub(super) mod system;

mod genapi_common;

use cameleon::ControlError;
use cameleon_impl::memory::MemoryError;

use super::GenTlError;

impl From<MemoryError> for GenTlError {
    fn from(err: MemoryError) -> Self {
        match err {
            MemoryError::AddressNotReadable | MemoryError::AddressNotWritable => Self::AccessDenied,
            MemoryError::InvalidAddress => Self::InvalidAddress,
            MemoryError::InvalidRegisterData(cause) => Self::InvalidValue(cause),
        }
    }
}

impl From<ControlError> for GenTlError {
    fn from(err: ControlError) -> Self {
        use GenTlError::{
            BufferTooSmall, InvalidValue, Io, NotInitialized, ResourceInUse, Timeout,
        };

        match err {
            ControlError::Busy => ResourceInUse,
            ControlError::Disconnected | ControlError::Io(..) | ControlError::InternalError(..) => {
                Io(err.into())
            }
            ControlError::NotOpened => NotInitialized,
            ControlError::InvalidData(..) => InvalidValue(format!("{}", err).into()),
            ControlError::Timeout => Timeout,
            ControlError::BufferTooSmall => BufferTooSmall,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CharEncoding {
    Ascii,
    UTF8,
}
