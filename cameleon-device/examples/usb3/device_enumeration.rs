extern crate cameleon_device;

use cameleon_device::usb3::*;

fn main() {
    // Build emulator in case build with emulator feature.
    #[cfg(feature = "emulator")]
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
