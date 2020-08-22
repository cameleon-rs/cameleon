use std::time::Duration;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use cameleon_device::usb3::{
    prelude::*,
    protocol::{ack, command},
    *,
};

const TIME_OUT: Duration = Duration::from_millis(100);

#[test]
fn test_exceptional_scenario() {
    // Set emulated device.
    DeviceBuilder::new().build();

    let mut devices = enumerate_device().unwrap();
    assert_eq!(devices.len(), 1);
    let device = devices.pop().unwrap();

    let mut control_channel = device.control_channel().unwrap();
    let mut control_channel2 = device.control_channel().unwrap();

    // The same channel can't open at the same time.
    control_channel.open().unwrap();
    assert! {
        match control_channel2.open()  {
            Err(Error::LibUsbError(LibUsbError::Busy)) => true,
            _ => false,
        }
    };

    // Trying to receive data without sending a command ends in timeout error.
    let mut buf = vec![0; 1024];
    assert! {
        match control_channel.recv(&mut buf, TIME_OUT) {
            Err(Error::LibUsbError(LibUsbError::Timeout)) => true,
            _ => false,
        }
    };

    // Trying to receive data with too small buffer ends in overflow error.

    // Write meaningless data.
    let dummy_data = &[1, 2, 3];
    assert! {control_channel.send(dummy_data, TIME_OUT).is_ok()};
    let mut buf = vec![0; 1];
    assert! {
        match control_channel.recv(&mut buf, TIME_OUT) {
            Err(Error::LibUsbError(LibUsbError::Overflow)) => true,
            _ => false,
        }
    };

    // Trying to use halted channel ends in pipe error.
    assert!(control_channel.set_halt(TIME_OUT).is_ok());
    assert! {
       match control_channel.send(dummy_data, TIME_OUT) {
           Err(Error::LibUsbError(LibUsbError::Pipe)) => true,
           _ => false,
       }
    };

    // Trying to use closed channel ends in io error.
    control_channel.close();
    assert! {
       match control_channel.send(dummy_data, TIME_OUT) {
           Err(Error::LibUsbError(LibUsbError::Io)) => true,
           _ => false,
       }
    };
}
