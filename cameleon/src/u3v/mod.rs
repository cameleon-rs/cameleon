//! This module provides low level API for U3V compatible devices.
//!
//! # Examples
//!
//! ```no_run
//! use cameleon::u3v;
//! // Enumerate devices connected to the host.
//! let mut devices = u3v::enumerate_devices().unwrap();
//!
//! // If no device is connected, return.
//! if devices.is_empty() {
//!     return;
//! }
//!
//! let device = devices.pop().unwrap();
//!
//! // Obtain and open the handle.
//! let handle = device.control_handle();
//! handle.open().unwrap();
//!
//! // Get Abrm.
//! let abrm = handle.abrm().unwrap();
//!
//! // Read serial number from ABRM.
//! let serial_number = abrm.serial_number().unwrap();
//! println!("{}", serial_number);
//!
//! // Check user defined name feature is supported.
//! // If it is suppoted, read from and write to the register.
//! let device_capability = abrm.device_capability().unwrap();
//! if device_capability.is_user_defined_name_supported() {
//!     // Read from user defined name register.
//!     let user_defined_name = abrm.user_defined_name().unwrap().unwrap();
//!     println!("{}", user_defined_name);
//!
//!     // Write new name to the register.
//!     abrm.set_user_defined_name("cameleon").unwrap();
//! }
//! ```
#![allow(clippy::missing_panics_doc)]

pub mod register_map;

pub mod control_handle;
pub mod device;
pub mod stream_handle;

pub use control_handle::ControlHandle;
pub use device::{enumerate_devices, Device, DeviceInfo};
pub use stream_handle::{StreamHandle, StreamParams};

use cameleon_device::u3v;

use super::DeviceError;

impl From<u3v::Error> for DeviceError {
    fn from(err: u3v::Error) -> DeviceError {
        use u3v::Error::{BufferIo, InvalidDevice, InvalidPacket, LibUsb};

        match &err {
            LibUsb(libusb_error) => {
                use u3v::LibUsbError::{
                    Access, BadDescriptor, Busy, Interrupted, InvalidParam, Io, NoDevice, NoMem,
                    NotFound, NotSupported, Other, Overflow, Pipe, Timeout,
                };
                match libusb_error {
                    Io | InvalidParam | Access | Overflow | Pipe | Interrupted | NoMem
                    | NotSupported | BadDescriptor | Other => DeviceError::Io(err.into()),
                    Busy => DeviceError::Busy,
                    NoDevice | NotFound => DeviceError::Disconnected,
                    Timeout => DeviceError::Timeout,
                }
            }

            BufferIo(_) | InvalidPacket(_) => DeviceError::Io(err.into()),

            InvalidDevice => panic!("device is broken"),
        }
    }
}
