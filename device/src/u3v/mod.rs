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

use nusb;
use nusb::transfer::TransferError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("usb error: {0}")]
    Usb(#[from] UsbError),

    #[error("packet is broken: {0}")]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error: {0}")]
    BufferIo(#[from] std::io::Error),

    #[error("device doesn't follow the specification")]
    InvalidDevice,
}

/// Errors raised from USB operations.
#[derive(Debug, Error)]
pub enum UsbError {
    #[error("transfer cancelled")]
    Cancelled,
    #[error("endpoint stalled")]
    Stall,
    #[error("device disconnected")]
    Disconnected,
    #[error("hardware fault or protocol violation")]
    Fault,
    #[error("invalid parameter")]
    InvalidParam,
    #[error("permission denied")]
    PermissionDenied,
    #[error("entity not found")]
    NotFound,
    #[error("resource busy")]
    Busy,
    #[error("operation timed out")]
    Timeout,
    #[error("operation not supported or unimplemented on this platform")]
    NotSupported,
    #[error("other error")]
    Other,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<nusb::Error> for Error {
    fn from(err: nusb::Error) -> Self {
        Error::Usb((&err).into())
    }
}

impl From<&nusb::Error> for UsbError {
    fn from(err: &nusb::Error) -> Self {
        use nusb::ErrorKind;
        match err.kind() {
            ErrorKind::Disconnected => UsbError::Disconnected,
            ErrorKind::Busy => UsbError::Busy,
            ErrorKind::PermissionDenied => UsbError::PermissionDenied,
            ErrorKind::NotFound => UsbError::NotFound,
            ErrorKind::Unsupported => UsbError::NotSupported,
            ErrorKind::Other => UsbError::Other,
            _ => UsbError::Other,
        }
    }
}

impl From<TransferError> for UsbError {
    fn from(err: TransferError) -> Self {
        match err {
            TransferError::Cancelled => UsbError::Cancelled,
            TransferError::Stall => UsbError::Stall,
            TransferError::Disconnected => UsbError::Disconnected,
            TransferError::Fault => UsbError::Fault,
            TransferError::InvalidArgument => UsbError::InvalidParam,
            TransferError::Unknown(_) => UsbError::Other,
        }
    }
}

impl From<TransferError> for Error {
    fn from(err: TransferError) -> Self {
        Error::Usb(err.into())
    }
}

impl From<nusb::GetDescriptorError> for Error {
    fn from(err: nusb::GetDescriptorError) -> Self {
        match err {
            nusb::GetDescriptorError::Transfer(inner) => Error::Usb(inner.into()),
            nusb::GetDescriptorError::InvalidDescriptor => Error::InvalidDevice,
        }
    }
}
