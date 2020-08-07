pub mod memory;

use thiserror::Error;

#[derive(Debug, Error)]
enum EmulatorError {
    #[error("attempt to access not existed memory location")]
    InvalidAddress,

    #[error("memory io error: {}", 0)]
    MemoryIoError(#[from] std::io::Error),

    #[error("invalid string: {}", 0)]
    InvalidString(&'static str),
}

type Result<T> = std::result::Result<T, EmulatorError>;
