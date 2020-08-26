#[cfg(not(feature = "libusb"))]
#[test]
fn test_exceptional_scenario() {
    use std::time::Duration;

    use cameleon_device::usb3::*;

    const TIME_OUT: Duration = Duration::from_millis(100);

    // Set emulated device.
    EmulatorBuilder::new().build();

    let mut devices = enumerate_device().unwrap();
    assert_eq!(devices.len(), 1);
    let device = devices.pop().unwrap();

    let mut control_channel = device.control_channel().unwrap();
    let mut control_channel2 = device.control_channel().unwrap();

    // The same channel can't open at the same time.
    control_channel.open().unwrap();
    assert! {
        matches!(control_channel2.open(), Err(Error::LibUsbError(LibUsbError::Busy)))
    };

    // Trying to receive data without sending a command ends in timeout error.
    let mut buf = vec![0; 1024];
    assert! {
        matches!(control_channel.recv(&mut buf, TIME_OUT), Err(Error::LibUsbError(LibUsbError::Timeout)))
    };

    // Trying to receive data with too small buffer ends in overflow error.

    // Write meaningless data.
    let dummy_data = &[1, 2, 3];
    assert! {control_channel.send(dummy_data, TIME_OUT).is_ok()};
    let mut buf = vec![0; 1];
    assert! {
        matches!(control_channel.recv(&mut buf, TIME_OUT), Err(Error::LibUsbError(LibUsbError::Overflow)))
    };

    // Trying to use halted channel ends in pipe error.
    assert!(control_channel.set_halt(TIME_OUT).is_ok());
    assert! {
       matches!(control_channel.send(dummy_data, TIME_OUT), Err(Error::LibUsbError(LibUsbError::Pipe)))
    };

    // Trying to use closed channel ends in io error.
    control_channel.close().unwrap();
    assert! {
       matches!(control_channel.send(dummy_data, TIME_OUT), Err(Error::LibUsbError(LibUsbError::Io)))
    };
}
