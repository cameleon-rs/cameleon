use thiserror::Error;

mod control_module;
mod fake_protocol;
mod interface;
mod memory;
mod signal;

#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("attempt to access not existed memory location")]
    InvalidAddress,

    #[error("invalid string: {}", 0)]
    InvalidString(&'static str),

    #[error("buffer io error in emulator: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device internal buffer is ful.")]
    FullBuffer,

    #[error("attempt to read unreadable address")]
    AddressNotReadable,

    #[error("attempt to write to unwritable address")]
    AddressNotWritable,
}

pub type EmulatorResult<T> = std::result::Result<T, EmulatorError>;
