pub(crate) use super::fake_protocol::IfaceKind;

use std::{
    io::Write,
    time::{Duration, Instant},
};

use async_std::{sync::TrySendError, task};

use crate::usb3::{LibUsbError, Result};

use super::{
    device_pool::{DevicePipe, DevicePool},
    fake_protocol::{FakeAckKind::*, FakeAckPacket, FakeReqKind, FakeReqPacket},
};

pub(crate) struct DeviceHandle {
    device_id: u32,
    channel: Option<DevicePipe>,
    iface_kind: IfaceKind,
}

impl DeviceHandle {
    pub(crate) fn new(device_id: u32, iface_kind: IfaceKind) -> Self {
        Self {
            device_id,
            channel: None,
            iface_kind,
        }
    }

    pub(crate) fn read_bulk(&self, mut buf: &mut [u8], timeout: Duration) -> Result<usize> {
        let start = Instant::now();

        while timeout.checked_sub(start.elapsed()).is_some() {
            let req = FakeReqPacket::new(self.iface_kind, FakeReqKind::Recv);
            let ack = self.send_packet(req)?;

            match ack.kind {
                RecvAck(data) => {
                    buf.write_all(&data).map_err(|_| LibUsbError::Overflow)?;
                    return Ok(data.len());
                }
                RecvNak => {
                    continue;
                }
                IfaceHalted => {
                    return Err(LibUsbError::Pipe.into());
                }
                _ => unreachable!(),
            }
        }

        Err(LibUsbError::Timeout.into())
    }

    pub(crate) fn write_bulk(&self, buf: &[u8], timeout: Duration) -> Result<usize> {
        let start = Instant::now();

        while timeout.checked_sub(start.elapsed()).is_some() {
            let req = FakeReqPacket::new(self.iface_kind, FakeReqKind::Send(buf.to_vec()));
            let ack = self.send_packet(req)?;

            match ack.kind {
                SendAck => {
                    return Ok(buf.len());
                }
                IfaceHalted => {
                    return Err(LibUsbError::Pipe.into());
                }
                _ => unreachable!(),
            }
        }

        Err(LibUsbError::Timeout.into())
    }

    pub(crate) fn set_halt(&self) -> Result<()> {
        let req = FakeReqPacket::new(self.iface_kind, FakeReqKind::SetHalt);
        let ack = self.send_packet(req)?;
        match ack.kind {
            SetHaltAck => Ok(()),
            _ => unreachable!(),
        }
    }

    pub(crate) fn clear_halt(&self) -> Result<()> {
        let req = FakeReqPacket::new(self.iface_kind, FakeReqKind::ClearHalt);
        let ack = self.send_packet(req)?;
        match ack.kind {
            ClearHaltAck => Ok(()),
            _ => unreachable!(),
        }
    }

    pub(crate) fn claim_interface(&mut self) -> Result<()> {
        if self.channel.is_some() {
            return Ok(());
        }

        let channel =
            DevicePool::with(|pool| pool.claim_interface(self.device_id, self.iface_kind))?;
        self.channel = Some(channel);
        Ok(())
    }

    pub(crate) fn release_interface(&mut self) -> Result<()> {
        if self.channel.is_none() {
            return Ok(());
        }

        DevicePool::with(|pool| pool.release_interface(self.device_id, self.iface_kind))?;
        self.channel = None;
        Ok(())
    }

    /// Attempt to send a request packet and receive an acknowledge packet from an emulated device.
    /// It's necessary to make send/recv operation atomic because fake protocol doesn't have state.
    /// An emulated device immediately returns acknowledge packet, so no need to worry about
    /// latency.
    fn send_packet(&self, packet: FakeReqPacket) -> Result<FakeAckPacket> {
        task::block_on(async {
            let channel = self.channel()?.lock().await;

            channel.0.try_send(packet).map_err(|err| match err {
                TrySendError::Disconnected(..) => LibUsbError::NoDevice,
                // This never occurs.
                TrySendError::Full(..) => unreachable!(),
            })?;

            let ack = channel.1.recv().await.map_err(|_| LibUsbError::NoDevice)?;

            debug_assert!(ack.iface == self.iface_kind);
            Ok(ack)
        })
    }

    fn channel(&self) -> Result<&DevicePipe> {
        match &self.channel {
            Some(channel) => Ok(channel),
            None => Err(LibUsbError::Io.into()),
        }
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        self.release_interface().ok();
    }
}
