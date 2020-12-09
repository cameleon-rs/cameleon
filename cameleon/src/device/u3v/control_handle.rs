use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cameleon_device::{
    u3v,
    u3v::protocol::{ack, cmd},
};

use super::register_map::Abrm;

use crate::device::{DeviceError, DeviceResult};

/// Initial timeout duration for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// Initial maximum command  packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_CMD_LENGTH: u32 = 128;

/// Initial maximum acknowledge packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_ACK_LENGTH: u32 = 128;

/// Thread safe control handle of the device.
#[derive(Clone)]
pub struct ControlHandle {
    inner: Arc<Mutex<ControlHandleImpl>>,
}

impl ControlHandle {
    /// Read data from the device's memory.
    /// Read length is same as `buf.len()`.
    pub fn read_mem(&self, address: u64, buf: &mut [u8]) -> DeviceResult<()> {
        self.inner.lock().unwrap().read_mem(address, buf)
    }

    /// Write data to the device's memory.
    pub fn write_mem(&self, address: u64, data: &[u8]) -> DeviceResult<()> {
        self.inner.lock().unwrap().write_mem(address, data)
    }

    /// Capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn buffer_capacity(&self) -> usize {
        self.inner.lock().unwrap().buffer_capacity()
    }

    /// Resize the capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn resize_buffer(&self, size: usize) {
        self.inner.lock().unwrap().resize_buffer(size)
    }

    /// Is control handle opened.
    pub fn is_opened(&self) -> bool {
        self.inner.lock().unwrap().is_opened()
    }

    /// Timeout duration of each transaction between device.
    /// NOTE: [`ControlHandle::read_mem`] and [`ControlHandle::write_mem`] may send multiple
    /// requests in a single call.
    pub fn timeout_duration(&self) -> Duration {
        self.inner.lock().unwrap().config.timeout_duration
    }

    /// Set timeout duration of each transaction between device.
    /// NOTE: [`ControlHandle::read_mem`] and [`ControlHandle::write_mem`] may send multiple
    /// requests in a single call.
    ///
    /// In normal use case, no need to modify timeout duration.
    pub fn set_timeout_duration(&self, duration: Duration) {
        self.inner.lock().unwrap().config.timeout_duration = duration;
    }

    /// The value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    pub fn retry_count(&self) -> u16 {
        self.inner.lock().unwrap().config.retry_count
    }

    pub fn abrm(&self) -> DeviceResult<Abrm> {
        Abrm::new(self)
    }

    /// Set the value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    pub fn set_retry_count(&mut self, count: u16) {
        self.inner.lock().unwrap().config.retry_count = count;
    }

    pub(super) fn open(&self) -> DeviceResult<()> {
        self.inner.lock().unwrap().open()?;
        self.initialize_config()
    }

    fn initialize_config(&self) -> DeviceResult<()> {
        let abrm = self.abrm()?;
        let sbrm = abrm.sbrm()?;

        let timeout_duration = abrm.maximum_device_response_time()?;
        let maximum_cmd_length = sbrm.maximum_command_transfer_length()?;
        let maximum_ack_length = sbrm.maximum_acknowledge_trasfer_length()?;

        let mut inner = self.inner.lock().unwrap();
        inner.config.timeout_duration = timeout_duration;
        inner.config.maximum_cmd_length = maximum_cmd_length;
        inner.config.maximum_ack_length = maximum_ack_length;

        Ok(())
    }

    pub(super) fn close(&self) -> DeviceResult<()> {
        self.inner.lock().unwrap().close()
    }

    pub(super) fn new(device: &u3v::Device) -> DeviceResult<Self> {
        let inner = ControlHandleImpl::new(device)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

struct ControlHandleImpl {
    inner: u3v::ControlChannel,

    config: ConnectionConfig,

    next_req_id: u16,

    /// Buffer for serializing/deserializing a packet.
    buffer: Vec<u8>,
}

impl ControlHandleImpl {
    /// Write data to the device's memory.
    fn write_mem(&mut self, address: u64, data: &[u8]) -> DeviceResult<()> {
        self.assert_open()?;

        let cmd = cmd::WriteMem::new(address, data)?;
        let maximum_cmd_length = self.config.maximum_cmd_length;

        for chunk in cmd.chunks(maximum_cmd_length as usize).unwrap() {
            let chunk_data_len = chunk.data_len();
            let ack: ack::WriteMem = self.send_cmd(chunk)?;

            if ack.length as usize != chunk_data_len {
                return Err(DeviceError::Io(
                    "write mem failed: written length mismatch".into(),
                ));
            }
        }

        Ok(())
    }

    /// Read data from the device's memory.
    /// Read length is same as `buf.len()`.
    fn read_mem(&mut self, mut address: u64, buf: &mut [u8]) -> DeviceResult<()> {
        self.assert_open()?;

        // Chunks buffer if buffer length is larger than u16::MAX.
        for buf_chunk in buf.chunks_mut(std::u16::MAX as usize) {
            // Create command for buffer chunk.
            let cmd = cmd::ReadMem::new(address, buf_chunk.len() as u16);
            let maximum_ack_length = self.config.maximum_ack_length;

            // Chunks command so that each acknowledge packet length fits to maximum_ack_length.
            let mut total_read_len = 0;
            for cmd_chunk in cmd.chunks(maximum_ack_length as usize).unwrap() {
                let read_len = cmd_chunk.read_length();
                let ack: ack::ReadMem = self.send_cmd(cmd_chunk)?;
                (&mut buf_chunk[total_read_len..total_read_len + read_len as usize])
                    .copy_from_slice(ack.data);
                total_read_len += read_len as usize;
            }

            address += buf_chunk.len() as u64;
        }

        Ok(())
    }

    /// Capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Resize the capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    fn resize_buffer(&mut self, size: usize) {
        self.buffer.resize(size, 0);
        self.buffer.shrink_to_fit();
    }

    fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.inner.open()?;
        // Clean up control channel state.
        self.inner.set_halt(self.config.timeout_duration)?;
        self.inner.clear_halt()?;

        Ok(())
    }

    fn assert_open(&self) -> DeviceResult<()> {
        if !self.is_opened() {
            Err(DeviceError::NotOpened)
        } else {
            Ok(())
        }
    }

    fn close(&mut self) -> DeviceResult<()> {
        if !self.is_opened() {
            Ok(())
        } else {
            Ok(self.inner.close()?)
        }
    }

    fn new(device: &u3v::Device) -> DeviceResult<Self> {
        let inner = device.control_channel()?;

        Ok(Self {
            inner,
            config: ConnectionConfig::default(),
            next_req_id: 0,
            buffer: Vec::new(),
        })
    }

    fn is_opened(&self) -> bool {
        self.inner.is_opened()
    }

    fn send_cmd<'a, T, U>(&'a mut self, cmd: T) -> DeviceResult<U>
    where
        T: cmd::CommandScd,
        U: ack::ParseScd<'a>,
    {
        let cmd = cmd.finalize(self.next_req_id);
        let cmd_len = cmd.cmd_len();
        let ack_len = cmd.maximum_ack_len();
        if self.buffer.len() < std::cmp::max(cmd_len, ack_len) {
            self.buffer.resize(std::cmp::max(cmd_len, ack_len), 0);
        }

        // Serialize and send command.
        cmd.serialize(self.buffer.as_mut_slice())?;
        self.inner
            .send(&self.buffer[..cmd_len], self.config.timeout_duration)?;

        // Receive ack.
        // If ack status is invalid, return error.
        // If ack status is valid and ack kind is pending, retry to receive ack up to retry count.
        // If ack status is valid and ack kind is not pending, try to interpret scd and return the
        // result.
        let mut retry_count = self.config.retry_count;
        let mut ok = None;
        while retry_count > 0 {
            let recv_len = self
                .inner
                .recv(&mut self.buffer, self.config.timeout_duration)?;

            let ack = ack::AckPacket::parse(&self.buffer[0..recv_len])?;
            self.verify_ack(&ack)?;

            if ack.scd_kind() == ack::ScdKind::Pending {
                let pending_ack: ack::Pending = ack.scd_as()?;
                std::thread::sleep(pending_ack.timeout);
                retry_count -= 1;
                continue;
            } else {
                self.next_req_id += 1;
                ok = Some(recv_len);
                break;
            }
        }

        // This codes seems weird due to a lifetime problem.
        // `ack::AckPacket::parse` is a fast operation, so it's ok to call it repeatedly.
        if let Some(recv_len) = ok {
            Ok(ack::AckPacket::parse(&self.buffer[0..recv_len])
                .unwrap()
                .scd_as()?)
        } else {
            Err(DeviceError::Io(
                "the number of times pending was returned exceeds the retry_count.".into(),
            ))
        }
    }

    fn verify_ack(&self, ack: &ack::AckPacket) -> DeviceResult<()> {
        let status = ack.status().kind();
        if status != ack::StatusKind::GenCp(ack::GenCpStatus::Success) {
            return Err(DeviceError::Io(
                format!("invalid status: {:?}", ack.status().kind()).into(),
            ));
        }

        if ack.request_id() != self.next_req_id {
            return Err(DeviceError::Io("request id mismatch".into()));
        }

        Ok(())
    }
}

struct ConnectionConfig {
    /// Timeout duration of each transaction between device.
    timeout_duration: Duration,

    /// The value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    retry_count: u16,

    /// Maximum length of a command sent to device from host. Unit is byte.
    maximum_cmd_length: u32,

    /// Maximum length of a acknowledge sent to host from device. Unit is byte.
    maximum_ack_length: u32,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout_duration: INITIAL_TIMEOUT_DURATION,
            retry_count: 3,
            maximum_cmd_length: INITIAL_MAXIMUM_CMD_LENGTH,
            maximum_ack_length: INITIAL_MAXIMUM_ACK_LENGTH,
        }
    }
}
