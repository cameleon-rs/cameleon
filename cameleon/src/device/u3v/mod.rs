pub mod register_map;

mod control_handle;
mod device;

pub use control_handle::ControlHandle;
pub use device::{enumerate_devices, Device, DeviceInfo};

use cameleon_device::u3v;

use super::DeviceError;

impl From<u3v::Error> for DeviceError {
    fn from(err: u3v::Error) -> DeviceError {
        use u3v::Error::*;

        match &err {
            LibUsbError(libusb_error) => {
                use u3v::LibUsbError::*;
                match libusb_error {
                    Io | InvalidParam | Access | Overflow | Pipe | Interrupted | NoMem
                    | NotSupported | BadDescriptor | Other => DeviceError::Io(err.into()),
                    Busy => DeviceError::Busy,
                    NoDevice | NotFound => DeviceError::Disconnected,
                    Timeout => DeviceError::Timeout,
                }
            }

            BufferIoError(_) | InvalidPacket(_) => DeviceError::Io(err.into()),

            InvalidDevice => panic!("device is broken"),
        }
    }
}
