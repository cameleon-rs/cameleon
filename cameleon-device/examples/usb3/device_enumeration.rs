extern crate cameleon_device;

use cameleon_device::usb3;

fn main() {
    let devices = usb3::enumerate_device().unwrap();
    if devices.is_empty() {
        println!("no device found");
    }

    for device in devices {
        println! {"{}", device.device_info()};
    }
}
