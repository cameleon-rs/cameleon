mod channel;
mod device;
mod device_builder;

pub use channel::{ControlChannel, ReceiveChannel};
pub use device::Device;
pub use device_builder::enumerate_devices;

use crate::u3v::{Error, LibUsbError};

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
