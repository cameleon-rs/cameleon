pub mod emulator;
pub mod protocol;
pub mod register_map;

mod device_info;

pub use device_info::{DeviceInfo, SupportedSpeed};

use std::{borrow::Cow, fmt};

use thiserror::Error;

#[cfg(feature = "libusb")]
mod real;
#[cfg(feature = "libusb")]
pub use real::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("libusb error: {}", 0)]
    LibusbError(LibUsbErrorKind),

    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device doesn't follow specification")]
    InvalidDevice,
}

/// Errors raised from libusb.
#[derive(Clone, Debug)]
pub enum LibUsbErrorKind {
    Io,
    InvalidParam,
    Access,
    NoDevice,
    NotFound,
    Busy,
    Timeout,
    Overflow,
    Pipe,
    Interrupted,
    NoMem,
    NotSupported,
    BadDescriptor,
    Other,
}

impl Into<Error> for LibUsbErrorKind {
    fn into(self) -> Error {
        Error::LibusbError(self)
    }
}

impl fmt::Display for LibUsbErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use LibUsbErrorKind::*;
        fmt.write_str(match self {
            Io => "input/output error",
            InvalidParam => "invalid parameter",
            Access => "access denied (insufficient permissions)",
            NoDevice => "no such device (it may have been disconnected)",
            NotFound => "entity not found",
            Busy => "resource busy",
            Timeout => "operation timed out",
            Overflow => "overflow",
            Pipe => "pipe error",
            Interrupted => "system call interrupted (perhaps due to signal)",
            NoMem => "insufficient memory",
            NotSupported => "operation not supported or unimplemented on this platform",
            BadDescriptor => "malformed descriptor",
            Other => "other error",
        })
    }
}

pub type Result<T> = std::result::Result<T, Error>;
