pub(crate) use super::fake_protocol::IfaceKind;

use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};

use async_std::{
    sync::{Mutex, Receiver, Sender, TrySendError},
    task,
};

use crate::usb3::{Error, LibUsbError, Result};

use super::{
    device_pool::DevicePool,
    fake_protocol::{FakeAckKind::*, FakeAckPacket, FakeReqKind, FakeReqPacket},
};

type DevicePipe = Arc<Mutex<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)>>;

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
                    return Err(LibUsbError::Pipe)?;
                }
                _ => unreachable!(),
            }
        }

        Err(LibUsbError::Timeout)?
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
                SendNak => {
                    continue;
                }
                IfaceHalted => {
                    return Err(LibUsbError::Pipe)?;
                }
                _ => unreachable!(),
            }
        }

        Err(LibUsbError::Timeout)?
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

    /// Attempt to send a request packet and receive acknowledge packet from an emulated device.
    /// It's necessary to send/recv operation atomic because fake protocol doesn't have state.
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
            None => Err(LibUsbError::Io)?,
        }
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        self.release_interface().ok();
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::sync::Mutex;

    use super::super::*;
    use super::*;

    fn build_device() {
        DeviceBuilder::new().build();
    }

    fn get_handle(iface: IfaceKind) -> DeviceHandle {
        let id = DevicePool::with(|pool| pool.device_ids())[0];
        DeviceHandle::new(id, iface)
    }

    fn disconnect(device_id: u32) {
        DevicePool::with(|pool| pool.disconnect(device_id));
    }

    lazy_static! {
        pub(crate) static ref TEST_GUARD: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_claim_interface() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();
        let mut ctrl_handle = get_handle(IfaceKind::Control);
        let mut stream_handle = get_handle(IfaceKind::Stream);

        // Claim interface.
        assert!(ctrl_handle.claim_interface().is_ok());

        // Can't claim interface if its already claimed.
        let mut ctrl_handle2 = get_handle(IfaceKind::Control);
        assert!(match ctrl_handle2.claim_interface() {
            Err(Error::LibUsbError(LibUsbError::Busy)) => true,
            _ => false,
        });

        // It's ok to calim another interface at the same time.
        assert!(stream_handle.claim_interface().is_ok());

        // It's ok to claim interface once it is released.
        assert!(ctrl_handle.release_interface().is_ok());
        assert!(ctrl_handle2.claim_interface().is_ok());

        disconnect(ctrl_handle.device_id)
    }

    #[test]
    fn test_write_bulk() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();
        let mut ctrl_handle = get_handle(IfaceKind::Control);
        ctrl_handle.claim_interface().unwrap();

        // Write meaningless data.
        let data = &[1, 2, 3];
        let timeout = Duration::from_millis(100);
        assert!(ctrl_handle.write_bulk(data, timeout).is_ok());

        disconnect(ctrl_handle.device_id)
    }

    #[test]
    fn test_read_bulk() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();
        let mut ctrl_handle = get_handle(IfaceKind::Control);
        ctrl_handle.claim_interface().unwrap();

        // Write meaningless data.
        let data = &[1, 2, 3];
        let timeout = Duration::from_millis(100);
        assert!(ctrl_handle.write_bulk(data, timeout).is_ok());

        // Read data.
        let mut buf = vec![0; 1024];
        assert!(ctrl_handle.read_bulk(&mut buf, timeout).is_ok());

        disconnect(ctrl_handle.device_id)
    }

    #[test]
    fn test_read_bulk_timeout() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();
        let mut ctrl_handle = get_handle(IfaceKind::Control);
        ctrl_handle.claim_interface().unwrap();

        let timeout = Duration::from_millis(100);
        let mut buf = vec![0; 1024];
        assert!(match ctrl_handle.read_bulk(&mut buf, timeout) {
            Err(Error::LibUsbError(LibUsbError::Timeout)) => true,
            other @ _ => panic!("{:?}", other),
        });

        disconnect(ctrl_handle.device_id)
    }

    #[test]
    fn test_overflow() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();
        let mut ctrl_handle = get_handle(IfaceKind::Control);
        ctrl_handle.claim_interface().unwrap();

        // Write meaningless data.
        let data = &[1, 2, 3];
        let timeout = Duration::from_millis(100);
        assert!(ctrl_handle.write_bulk(data, timeout).is_ok());

        // Read data.
        let mut buf = vec![];
        assert!(match ctrl_handle.read_bulk(&mut buf, timeout) {
            Err(Error::LibUsbError(LibUsbError::Overflow)) => true,
            _ => false,
        });

        disconnect(ctrl_handle.device_id)
    }

    #[test]
    fn test_use_no_claimed_iface() {
        let _lock = TEST_GUARD.lock().unwrap();

        build_device();

        let ctrl_handle = get_handle(IfaceKind::Control);
        let timeout = Duration::from_millis(100);
        let mut buf = vec![0; 1024];
        assert!(match ctrl_handle.read_bulk(&mut buf, timeout) {
            Err(Error::LibUsbError(LibUsbError::Io)) => true,
            other @ _ => panic!("{:?}", other),
        });

        disconnect(ctrl_handle.device_id)
    }
}
