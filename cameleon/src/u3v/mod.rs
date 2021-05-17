//! This module provides low level API for U3V compatible devices.
//!
//! # Examples
//!
//! ```no_run
//! use cameleon::Camera;
//! use cameleon::u3v::{self, U3VDeviceControl};
//! use cameleon::genapi;
//!
//! // Enumerates cameras connected to the host.
//! let mut cameras = u3v::enumerate_cameras().unwrap();
//!
//! // If no camera is found, return.
//! if cameras.is_empty() {
//!     return;
//! }
//!
//! let mut camera = cameras.pop().unwrap();
//! // Opens the camera.
//! camera.open();
//!
//! let ctrl = &mut camera.ctrl;
//! // Get Abrm.
//! let abrm = ctrl.abrm().unwrap();
//!
//! // Read serial number from ABRM.
//! let serial_number = abrm.serial_number(ctrl).unwrap();
//! println!("{}", serial_number);
//!
//! // Check user defined name feature is supported.
//! // If it is suppoted, read from and write to the register.
//! let device_capability = abrm.device_capability().unwrap();
//! if device_capability.is_user_defined_name_supported() {
//!     // Read from user defined name register.
//!     let user_defined_name = abrm.user_defined_name(ctrl).unwrap().unwrap();
//!     println!("{}", user_defined_name);
//!
//!     // Write new name to the register.
//!     abrm.set_user_defined_name(ctrl, "cameleon").unwrap();
//! }
//! ```
#![allow(clippy::missing_panics_doc)]

pub mod register_map;

pub mod control_handle;
pub mod stream_handle;

pub use control_handle::{ControlHandle, SharedControlHandle};
pub use stream_handle::{StreamHandle, StreamParams};

pub use cameleon_device::u3v::DeviceInfo;

use cameleon_device::u3v;

use super::{genapi::DefaultGenApiCtxt, CameleonResult, Camera, CameraInfo, ControlError};

/// Enumerate all U3V compatible cameras connected to the host.
///
/// # Examples
///
/// ```no_run
/// use cameleon::Camera;
/// use cameleon::u3v;
/// use cameleon::genapi;
///
/// // Enumerate cameras connected to the host.
/// let mut cameras = u3v::enumerate_cameras().unwrap();
/// ```
pub fn enumerate_cameras() -> CameleonResult<Vec<Camera<ControlHandle, StreamHandle>>> {
    let devices = u3v::enumerate_devices().map_err(ControlError::from)?;

    let mut cameras: Vec<Camera<ControlHandle, StreamHandle>> = Vec::with_capacity(devices.len());

    for dev in devices {
        let ctrl = ControlHandle::new(&dev)?;
        let strm = if let Some(strm) = StreamHandle::new(&dev)? {
            strm
        } else {
            continue;
        };
        let ctxt = None;

        let dev_info = dev.device_info;
        let camera_info = CameraInfo {
            vendor_name: dev_info.vendor_name,
            model_name: dev_info.model_name,
            serial_number: dev_info.serial_number,
        };

        let camera: Camera<ControlHandle, StreamHandle, DefaultGenApiCtxt> =
            Camera::new(ctrl, strm, ctxt, camera_info);
        cameras.push(camera)
    }

    Ok(cameras)
}

impl From<u3v::Error> for ControlError {
    fn from(err: u3v::Error) -> ControlError {
        use u3v::Error::{BufferIo, InvalidDevice, InvalidPacket, LibUsb};

        match &err {
            LibUsb(libusb_error) => {
                use u3v::LibUsbError::{
                    Access, BadDescriptor, Busy, Interrupted, InvalidParam, Io, NoDevice, NoMem,
                    NotFound, NotSupported, Other, Overflow, Pipe, Timeout,
                };
                match libusb_error {
                    Io | InvalidParam | Access | Overflow | Pipe | Interrupted | NoMem
                    | NotSupported | BadDescriptor | Other => ControlError::Io(err.into()),
                    Busy => ControlError::Busy,
                    NoDevice | NotFound => ControlError::Disconnected,
                    Timeout => ControlError::Timeout,
                }
            }

            BufferIo(_) | InvalidPacket(_) => ControlError::Io(err.into()),

            InvalidDevice => panic!("device is broken"),
        }
    }
}
