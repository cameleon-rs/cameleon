use std::ffi::CStr;
use std::time::Duration;

extern crate byteorder;
extern crate cameleon_device;

use cameleon_device::usb3;

fn main() {
    usb3::DeviceBuilder::new().build();

    // Enumerate device connected to the host.
    let devices: Vec<usb3::Device> = usb3::enumerate_device().unwrap().into_iter().collect();

    if devices.is_empty() {
        println!("no device found");
    }

    for device in devices {
        println! {"{}", device.device_info()};
    }
}
