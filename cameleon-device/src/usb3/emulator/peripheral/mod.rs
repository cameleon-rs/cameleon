pub mod device_builder;

mod control_module;
mod device;
mod device_pool;
mod event_module;
mod fake_protocol;
mod interface;
mod memory;
mod signal;
mod stream_module;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("invalid string: {}", 0)]
    InvalidString(&'static str),

    #[error("buffer io error in emulator: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device internal buffer is ful.")]
    FullBuffer,
}

pub type EmulatorResult<T> = std::result::Result<T, EmulatorError>;
