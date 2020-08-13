use std::borrow::Cow;

use thiserror::Error;

mod memory;
mod memory_protection;
mod protocol;

#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("attempt to access not existed memory location")]
    InvalidAddress,

    #[error("invalid string: {}", 0)]
    InvalidString(&'static str),

    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error in emulator: {}", 0)]
    BufferIoError(#[from] std::io::Error),
}

pub type EmulatorResult<T> = std::result::Result<T, EmulatorError>;
