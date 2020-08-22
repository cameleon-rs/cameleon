use std::time::Duration;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use cameleon_device::usb3::{
    protocol::{ack, command},
    *,
};

const TIME_OUT: Duration = Duration::from_millis(100);

#[test]
fn test_normal_scenario() {
    // Set emulated device.
    DeviceBuilder::new().build();

    let mut devices = enumerate_device().unwrap();
    assert_eq!(devices.len(), 1);

    let device = devices.pop().unwrap();

    let mut control_channel = device.control_channel().unwrap();

    // An emulator has both event and stream channels.
    let mut event_channel = device.event_channel().unwrap().unwrap();
    let mut stream_channel = device.stream_channel().unwrap().unwrap();

    // Open channel to communicate with the device.
    assert!(control_channel.open().is_ok());
    assert!(event_channel.open().is_ok());
    assert!(stream_channel.open().is_ok());

    let mut req_id = 0;

    // Send WriteMem command to time stamp latch entry of the device.
    // The command will dispatch a internal event which cause write to time stamp entry of the
    // device.
    let (tsl_addr, _, _) = register_map::abrm::TIMESTAMP_LATCH;
    let cmd_data = u32_as_le_bytes(1);
    let (write_cmd, ack_len) = write_cmd(tsl_addr, &cmd_data, req_id);
    assert!(control_channel.send(&write_cmd, TIME_OUT).is_ok());

    // Receive acknowledge packet corresponding to WriteMem command sent above.
    let mut ack_bytes = vec![0; ack_len];
    assert!(control_channel.recv(&mut ack_bytes, TIME_OUT).is_ok());
    let ack_command = ack::AckPacket::parse(&ack_bytes).unwrap();

    assert_eq!(ack_command.request_id(), req_id);
    assert!(ack_command.status().is_success());
    let write_ack_scd: ack::WriteMem = ack_command.scd_as().unwrap();
    assert_eq!(write_ack_scd.length as usize, cmd_data.len());

    // Increment req_id for next command.
    req_id += 1;

    // Send ReadMem command to time stamp entry.
    let (ts_addr, ts_len, _) = register_map::abrm::TIMESTAMP;
    let (cmd, ack_len) = read_cmd(ts_addr, ts_len, req_id);
    assert!(control_channel.send(&cmd, TIME_OUT).is_ok());

    // Receive acknowledge packet corresponding to ReadMem command sent above.
    let mut ack_bytes = vec![0; ack_len];
    assert!(control_channel.recv(&mut ack_bytes, TIME_OUT).is_ok());
    let ack_command = ack::AckPacket::parse(&ack_bytes).unwrap();

    assert_eq!(ack_command.request_id(), req_id);
    assert!(ack_command.status().is_success());
    let read_ack_command: ack::ReadMem = ack_command.scd_as().unwrap();
    assert_eq!(read_ack_command.data.len(), ts_len as usize);

    // Assert time stamp is larger than zero, it verify write to time stamp latch works correctly.
    let time_stamp = bytes_as_u64_le(&read_ack_command.data);
    assert!(time_stamp > 0);

    // Increment req_id for next command.
    req_id += 1;

    // Send ReadMem command to time stamp entry.
    let (ts_addr, ts_len, _) = register_map::abrm::TIMESTAMP;
    let (cmd, _) = read_cmd(ts_addr, ts_len, req_id);
    assert!(control_channel.send(&cmd, TIME_OUT).is_ok());

    // Assert set/clear halt works.
    assert!(control_channel.set_halt(TIME_OUT).is_ok());
    assert!(control_channel.clear_halt().is_ok());

    // Assert control channel is empty after halt.
    assert! {
       match control_channel.recv(&mut [], TIME_OUT) {
           Err(Error::LibUsbError(LibUsbError::Timeout)) => true,
           _ => false
       }
    };
}

fn bytes_as_u64_le(mut bytes: &[u8]) -> u64 {
    bytes.read_u64::<LE>().unwrap()
}

fn u32_as_le_bytes(num: u32) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.write_u32::<LE>(num).unwrap();
    bytes
}

fn write_cmd(addr: u64, data: &[u8], req_id: u16) -> (Vec<u8>, usize) {
    let cmd = command::WriteMem::new(addr, &data)
        .unwrap()
        .finalize(req_id);

    let mut bytes = Vec::new();
    cmd.serialize(&mut bytes).unwrap();
    (bytes, cmd.maximum_ack_len().unwrap())
}

fn read_cmd(addr: u64, len: u16, req_id: u16) -> (Vec<u8>, usize) {
    let cmd = command::ReadMem::new(addr, len).finalize(req_id);

    let mut bytes = Vec::new();
    cmd.serialize(&mut bytes).unwrap();
    (bytes, cmd.maximum_ack_len().unwrap())
}
