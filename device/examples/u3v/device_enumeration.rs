extern crate cameleon_device;

use cameleon_device::u3v::enumerate_devices;

fn main() {
    let devices = enumerate_devices().unwrap();
    if devices.is_empty() {
        println!("no device found");
    }

    for device in devices {
        println! {"{}", device.device_info};
    }
}
