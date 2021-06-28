/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod async_read;
pub mod protocol;
pub mod register_map;
pub mod prelude {
    pub use protocol::ack::ParseScd;
    pub use protocol::cmd::CommandScd;

    use super::protocol;
}

mod channel;
mod device;
mod device_builder;
mod device_info;

pub use channel::{ControlChannel, ReceiveChannel};
pub use device::Device;
pub use device_builder::enumerate_devices;
pub use device_info::{BusSpeed, DeviceInfo};

use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("libusb error: {0}")]
    LibUsb(#[from] LibUsbError),

    #[error("packet is broken: {0}")]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error: {0}")]
    BufferIo(#[from] std::io::Error),

    #[error("device doesn't follow the specification")]
    InvalidDevice,
}

/// Errors raised from libusb.
#[derive(Debug, Error)]
pub enum LibUsbError {
    #[error("input/output error")]
    Io,
    #[error("invalid parameter")]
    InvalidParam,
    #[error("access denied (insufficient permissions)")]
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

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Error {
        use LibUsbError::{
            Access, BadDescriptor, Busy, Interrupted, InvalidParam, Io, NoDevice, NoMem, NotFound,
            NotSupported, Other, Overflow, Pipe, Timeout,
        };
        let kind = match err {
            rusb::Error::Io => Io,
            rusb::Error::InvalidParam => InvalidParam,
            rusb::Error::Access => Access,
            rusb::Error::NoDevice => NoDevice,
            rusb::Error::NotFound => NotFound,
            rusb::Error::Busy => Busy,
            rusb::Error::Timeout => Timeout,
            rusb::Error::Overflow => Overflow,
            rusb::Error::Pipe => Pipe,
            rusb::Error::Interrupted => Interrupted,
            rusb::Error::NoMem => NoMem,
            rusb::Error::NotSupported => NotSupported,
            rusb::Error::BadDescriptor => BadDescriptor,
            rusb::Error::Other => Other,
        };

        Error::LibUsb(kind)
    }
}
