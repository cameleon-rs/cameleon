extern crate cameleon_device;

use cameleon_device::u3v::*;

fn main() {
    // Need to build emulator in case libusb is not supported.
    #[cfg(not(feature = "libusb"))]
    EmulatorBuilder::new()
        .user_defined_name("emu")
        .unwrap()
        .build();

    let devices = enumerate_device().unwrap();
    if devices.is_empty() {
        println!("no device found");
    }

    for device in devices {
        println! {"{}", device.device_info()};
    }
}
