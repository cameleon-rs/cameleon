use std::{
    sync::{Arc, Mutex, MutexGuard},
    time,
};

use cameleon_device::u3v::{
    self,
    protocol::stream::{Leader, Trailer},
};

use crate::device::{DeviceError, DeviceResult};

use super::register_map::Sirm;

/// This type is used to receive stream packets from the device.
///
/// Parameters required to receive a packet is inferred from SIRM, but please make sure the
/// parameters are set to right values before starting to call `read_*` methods.
#[derive(Clone)]
pub struct StreamHandle {
    channel: Arc<Mutex<u3v::ReceiveChannel>>,
    params: StreamParams,
}

impl StreamHandle {
    /// Open the handle.
    pub fn open(&self) -> DeviceResult<()> {
        Ok(self.channel.lock().unwrap().open()?)
    }

    /// Close the handle.
    pub fn close(&self) -> DeviceResult<()> {
        Ok(self.channel.lock().unwrap().close()?)
    }

    /// Return `true` if the handle is opened.
    pub fn is_opened(&self) -> bool {
        self.channel.lock().unwrap().is_opened()
    }

    /// Read leader of a stream packet.
    ///
    /// Buffer size must be equal or larger than [`StreamParams::leader_size`].
    pub fn read_leader<'a>(
        &self,
        buf: &'a mut [u8],
        timeout: time::Duration,
    ) -> DeviceResult<Leader<'a>> {
        let leader_size = self.params.leader_size;
        Self::recv(&self.channel.lock().unwrap(), buf, leader_size, timeout)?;

        Ok(Leader::parse(buf)?)
    }

    /// Read payload of a  stream packet.
    pub fn read_payload(&self, buf: &mut [u8], timeout: time::Duration) -> DeviceResult<usize> {
        let guard = self.channel.lock().unwrap();
        let mut read_len = 0;

        let payload_size = self.params.payload_size;
        for _ in 0..self.params.payload_count {
            read_len += Self::recv(
                &guard,
                &mut buf[read_len..read_len + payload_size],
                payload_size,
                timeout,
            )?;
        }

        let payload_final1_size = self.params.payload_final1_size;
        read_len += Self::recv(
            &guard,
            &mut buf[read_len..read_len + payload_final1_size],
            payload_final1_size,
            timeout,
        )?;

        let payload_final2_size = self.params.payload_final2_size;
        read_len += Self::recv(
            &guard,
            &mut buf[read_len..read_len + payload_final2_size],
            payload_final2_size,
            timeout,
        )?;

        Ok(read_len)
    }

    /// Read trailer of a stream packet.
    ///
    /// Buffer size must be equal of larger than [`StreamParams::trailer_size`].
    pub fn read_trailer<'a>(
        &self,
        buf: &'a mut [u8],
        timeout: time::Duration,
    ) -> DeviceResult<Trailer<'a>> {
        let trailer_size = self.params.trailer_size as usize;
        Self::recv(&self.channel.lock().unwrap(), buf, trailer_size, timeout)?;

        Ok(Trailer::parse(buf)?)
    }

    /// Return params.
    pub fn params(&self) -> &StreamParams {
        &self.params
    }

    ///  Return mutable params.
    pub fn params_mut(&mut self) -> &mut StreamParams {
        &mut self.params
    }

    pub(super) fn new(device: &u3v::Device) -> DeviceResult<Option<Self>> {
        let channel = device.stream_channel()?;
        if let Some(channel) = channel {
            Ok(Some(Self {
                channel: Arc::new(Mutex::new(channel)),
                params: Default::default(),
            }))
        } else {
            Ok(None)
        }
    }

    fn recv(
        channel_guard: &MutexGuard<'_, u3v::ReceiveChannel>,
        buf: &mut [u8],
        len: usize,
        timeout: time::Duration,
    ) -> DeviceResult<usize> {
        if len == 0 {
            return Ok(0);
        }

        if buf.len() < len {
            return Err(DeviceError::BufferTooSmall);
        }

        Ok(channel_guard.recv(&mut buf[..len], timeout)?)
    }
}

/// Parameters to receive stream packets.
///
/// Both `StreamParams` and [`StreamHandle`] don't check integrity of the paremter. That's up to user.
#[derive(Debug, Clone, Default)]
pub struct StreamParams {
    /// Maximum leader size.
    pub leader_size: usize,

    /// Maximum trailer size.
    pub trailer_size: usize,

    /// Payload transfer size.
    pub payload_size: usize,

    /// Payload transfer count.
    pub payload_count: usize,

    /// Payload transfer final1 size.
    pub payload_final1_size: usize,

    /// Payload transfer final2 size.
    pub payload_final2_size: usize,
}

impl StreamParams {
    /// Constructor of `StreamParams`.
    pub fn new(
        leader_size: usize,
        trailer_size: usize,
        payload_size: usize,
        payload_count: usize,
        payload_final1_size: usize,
        payload_final2_size: usize,
    ) -> Self {
        Self {
            leader_size,
            trailer_size,
            payload_size,
            payload_count,
            payload_final1_size,
            payload_final2_size,
        }
    }

    /// Build `StreamParams` from [`Sirm`].
    pub fn from_sirm(sirm: &Sirm<'_>) -> DeviceResult<Self> {
        let leader_size = sirm.maximum_leader_size()? as usize;
        let trailer_size = sirm.maximum_trailer_size()? as usize;

        let payload_size = sirm.payload_transfer_size()? as usize;
        let payload_count = sirm.payload_transfer_count()? as usize;
        let payload_final1_size = sirm.payload_final_transfer1_size()? as usize;
        let payload_final2_size = sirm.payload_final_transfer2_size()? as usize;

        Ok(Self::new(
            leader_size,
            trailer_size,
            payload_size,
            payload_count,
            payload_final1_size,
            payload_final2_size,
        ))
    }
}
