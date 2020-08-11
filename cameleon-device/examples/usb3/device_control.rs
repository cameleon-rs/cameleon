use std::ffi::CStr;
use std::time::Duration;

extern crate byteorder;
extern crate cameleon_device;

use cameleon_device::usb3::{
    self,
    protocol::{ack, command},
};

fn main() {
    // Enumerate device connected to the host.
    let devices: Vec<usb3::Device> = usb3::enumerate_device().unwrap().into_iter().collect();

    if devices.is_empty() {
        println!("no device found");
        return;
    }

    let device = &devices[0];

    // Get control channel of the device.
    let mut control_channel = device.control_channel().unwrap();

    // Open the channel to allow communication with the device.
    control_channel.open().unwrap();

    // TODO: Need helper for accessing BRM.
    // TODO: Get Maximum ack length from command.
    // TODO: Add description about request_id.
    let command = command::ReadMem::new(0x00144, 64).finalize(0);

    let mut serialized_command = vec![];
    command.serialize(&mut serialized_command).unwrap();

    //  Send read request to the device.
    control_channel
        .send(&serialized_command, Duration::from_millis(100))
        .unwrap();

    // Receive Acknowledge packet from device.
    let mut serialized_ack = vec![0; command.maximum_ack_len().unwrap()];
    control_channel
        .recv(&mut serialized_ack, Duration::from_millis(100))
        .unwrap();
    // Parse Acknowledge packet.
    let ack = ack::AckPacket::parse(&serialized_ack).unwrap();

    // Check status and request_id.
    if !ack.status().is_success() || ack.request_id() != 0 {
        println!("Invalid acknowledge packet!");
        return;
    }

    let scd = ack.scd_as::<ack::ReadMem>().unwrap();
    let string_len = scd.data.iter().position(|c| *c == 0).unwrap();
    let serial_number = CStr::from_bytes_with_nul(&scd.data[..string_len + 1]).unwrap(); // Zero terminated.

    println!(
        "received serial number of the device: {}",
        serial_number.to_str().unwrap()
    );
}

// TODO: Add DeviceContext for example.
