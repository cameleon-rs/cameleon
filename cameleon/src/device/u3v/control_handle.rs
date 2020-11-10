use std::time::Duration;

use cameleon_device::{
    u3v,
    u3v::protocol::{ack, cmd},
};

use crate::device::{DeviceError, DeviceResult};

/// Initial timeout duration for transaction between device and host.
const INITIAL_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// Initial maximum command  packet length for transaction between device and host.
const INITIAL_MAXIMUM_CMD_LENGTH: u32 = 1024;

/// Initial maximum acknowledge packet length for transaction between device and host.
const INITIAL_MAXIMUM_ACK_LENGTH: u32 = 1024;

pub struct ControlHandle {
    inner: u3v::ControlChannel,

    config: ConnectionConfig,

    next_req_id: u16,

    /// Buffer for serializing/deserializing a packet.
    buffer: Vec<u8>,
}

impl ControlHandle {
    /// Set configuration that manages connection parameters.
    /// NOTE: The parameter is shared by all handles provided by the same [`super::Device`].
    pub fn set_config(&mut self, config: ConnectionConfig) {
        self.config = config;
    }

    pub fn write_mem(&mut self, address: u64, data: &[u8]) -> DeviceResult<()> {
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

    pub fn read_mem(&mut self, mut address: u64, buf: &mut [u8]) -> DeviceResult<()> {
        // Chunks buffer if buffer length is larger than u16::MAX.
        for buf_chunk in buf.chunks_mut(std::u16::MAX as usize) {
            // Create command for buffer chunk.
            let cmd = cmd::ReadMem::new(address, buf_chunk.len() as u16);
            let maximum_ack_length = self.config.maximum_ack_length;

            // Chunks command so that acknowledge packet length fits to maximum_ack_length.
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

    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    pub fn shrink_buffer(&mut self, size: usize) {
        self.buffer.resize(size, 0);
        self.buffer.shrink_to_fit();
    }

    pub(super) fn open(&mut self) -> DeviceResult<()> {
        self.inner.open()?;
        // Clean up control channel state.
        self.inner.set_halt(self.config.timeout_duration())?;
        self.inner.clear_halt()?;

        Ok(())
    }

    pub(super) fn close(&mut self) -> DeviceResult<()> {
        Ok(self.inner.close()?)
    }

    pub(super) fn new(device: &u3v::Device) -> DeviceResult<Self> {
        let inner = device.control_channel()?;

        Ok(Self {
            inner,
            config: ConnectionConfig::default(),
            next_req_id: 0,
            buffer: Vec::new(),
        })
    }

    pub(super) fn is_opened(&self) -> bool {
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
            .send(&self.buffer[..cmd_len], self.config.timeout_duration())?;

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
                .recv(&mut self.buffer, self.config.timeout_duration())?;

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

pub struct ConnectionConfig {
    timeout_duration: Duration,

    retry_count: u16,

    /// Maximum length of a command sent to device from host. Unit is byte.
    maximum_cmd_length: u32,

    /// Maximum length of a ack sent to host from device. Unit is byte.
    maximum_ack_length: u32,
}

impl ConnectionConfig {
    /// Timeout duration of each transaction between device.
    pub fn timeout_duration(&self) -> Duration {
        self.timeout_duration
    }

    /// Set timeout duration of each transaction between device.
    pub fn set_timeout_duration(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }
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
