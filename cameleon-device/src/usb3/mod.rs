pub mod emulator;
pub mod protocol;
pub mod register_map;

mod device_info;

pub use device_info::{DeviceInfo, SupportedSpeed};

use std::borrow::Cow;

use thiserror::Error;

#[cfg(feature = "libusb")]
mod real;
#[cfg(feature = "libusb")]
pub use real::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("libusb error: {}", 0)]
    LibUsbError(#[from] LibUsbError),

    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device doesn't follow specification")]
    InvalidDevice,
}

/// Errors raised from libusb.
#[derive(Debug, Error)]
pub enum LibUsbError {
    #[error("input/output error")]
    Io,
    #[error("invalid parameter")]
    InvalidParam,
    #[error("access denied (insufficient permissins)")]
    Access,
    #[error("no such device (it may have been disconnected)")]
    NoDevice,
    #[error("entity not found")]
    NotFound,
    #[error("resource busy")]
    Busy,
    #[error("operation timed out")]
    Timeout,
    #[error("overflow")]
    Overflow,
    #[error("pipe error")]
    Pipe,
    #[error("system call interrupted (perhaps due to signal)")]
    Interrupted,
    #[error("insufficient memory")]
    NoMem,
    #[error("operation not supported or unimplemented on this platform")]
    NotSupported,
    #[error("malformed descriptor")]
    BadDescriptor,
    #[error("other error")]
    Other,
}

pub type Result<T> = std::result::Result<T, Error>;
