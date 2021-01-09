pub(super) mod device;
pub(super) mod interface;
pub(super) mod port;
pub(super) mod system;

mod genapi_common;

use cameleon::device::DeviceError;
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

impl From<DeviceError> for GenTlError {
    fn from(err: DeviceError) -> Self {
        use GenTlError::*;

        match err {
            DeviceError::Busy => ResourceInUse,
            DeviceError::Disconnected | DeviceError::Io(..) | DeviceError::InternalError(..) => {
                Io(err.into())
            }
            DeviceError::NotOpened => NotInitialized,
            DeviceError::InvalidData(..) => InvalidValue(format!("{}", err).into()),
            DeviceError::Timeout => Timeout,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CharEncoding {
    Ascii,
    UTF8,
}
