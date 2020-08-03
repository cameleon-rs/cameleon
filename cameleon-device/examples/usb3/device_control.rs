use std::ffi::CStr;
use std::time::Duration;

extern crate byteorder;
extern crate cameleon_device;

use cameleon_device::usb3;

fn main() {
    // Enumerate device connected to the host.
    let devices: Vec<usb3::Device> = usb3::enumerate_device()
        .unwrap()
        .into_iter()
        .filter_map(|device| device.ok())
        .collect();

    if devices.is_empty() {
        println!("no device found");
        return;
    }

    let device = &devices[0];

    // Get control handle of the device.
    let mut control_handle = device.control_handle().unwrap();

    // Open the handle to allow communication with the device.
    control_handle.open().unwrap();

    // TODO: Need helper for accessing BRM.
    // TODO: Get Maximum ack length from command.
    // TODO: Add description about request_id.
    let command = usb3::protocol::ReadMem::new(0x00144, 64).finalize(0);

    let mut serialized_command = vec![];
    command.serialize(&mut serialized_command).unwrap();

    //  Send read request to the device.
    control_handle
        .write(&serialized_command, Duration::from_millis(100))
        .unwrap();

    // Receive Acknowledge packet from device.
    let mut serialized_ack = vec![0; command.maximum_ack_len().unwrap()];
    control_handle
        .read(&mut serialized_ack, Duration::from_millis(100))
        .unwrap();
    // Parse Acknowledge packet.
    let ack = usb3::protocol::AckPacket::parse(&serialized_ack).unwrap();

    // Check status and request_id.
    if !ack.status().is_success() || ack.request_id() != 0 {
        println!("Invalid acknowledge packet!");
        return;
    }

    let serial_number = match ack.scd.unwrap() {
        usb3::protocol::AckScd::ReadMem { data } => {
            let string_len = data.iter().position(|c| *c == 0).unwrap();
            CStr::from_bytes_with_nul(&data[..string_len + 1]).unwrap() // Zero terminated.
        }
        _ => {
            println!("Invalid command data!");
            return;
        }
    };

    println!(
        "received serial number of the device: {}",
        serial_number.to_str().unwrap()
    );
}

// TODO: Add DeviceContext for example.
