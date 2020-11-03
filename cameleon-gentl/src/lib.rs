pub mod interface;
pub mod port;
pub mod system;

use thiserror::Error;

use cameleon_impl::memory::MemoryError;

#[derive(Error, Debug)]
pub enum GenTlError {
    /// The handle isn't opend.
    #[error("the handle isn't opend")]
    NotOpend,

    /// The access to the requested register address is denied because the register is not writable
    /// or because the Port module is opened in a way that it does not allow write access.
    #[error("the access to the requested register addresss is denied")]
    AccessDenied,

    /// There is no register with the provided address.
    #[error("there is no register with the provided address")]
    InvalidAddress,

    /// An invalid value has been written.
    #[error("An invalid value has been written: {}", 0)]
    InvalidValue(std::borrow::Cow<'static, str>),

    /// Communication error or connection lost.
    #[error("Communication error or connection lost: {}", 0)]
    IoError(Box<dyn std::error::Error>),
}

impl From<MemoryError> for GenTlError {
    fn from(err: MemoryError) -> Self {
        match err {
            MemoryError::AddressNotReadable | MemoryError::AddressNotWritable => Self::AccessDenied,
            MemoryError::InvalidAddress => Self::InvalidAddress,
            MemoryError::InvalidRegisterData(cause) => Self::InvalidValue(cause),
        }
    }
}

pub type GenTlResult<T> = std::result::Result<T, GenTlError>;
