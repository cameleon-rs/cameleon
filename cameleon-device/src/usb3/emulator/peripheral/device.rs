use std::{convert::TryInto, sync::Arc, time};

use async_std::{
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use futures::channel::oneshot;

use super::{fake_protocol::*, interface::Interface, memory::Memory};

const REQ_PACKET_CHANNEL_CAPACITY: usize = 1;
const ACK_PACKET_CHANNEL_CAPACITY: usize = 1;

pub(super) struct Device {
    timestamp: Timestamp,
    memory: Arc<Mutex<Memory>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    completion_rx: Option<oneshot::Receiver<()>>,
}

impl Device {
    pub(super) fn new(memory: Memory) -> Self {
        Self {
            timestamp: Timestamp::new(),
            memory: Arc::new(Mutex::new(memory)),
            shutdown_tx: None,
            completion_rx: None,
        }
    }

    pub(super) fn run(&mut self) -> (Sender<FakeReqPacket>, Receiver<FakeAckPacket>) {
        // Create channels for communication between device and host.
        let (req_tx_for_host, req_rx_for_device) = channel(REQ_PACKET_CHANNEL_CAPACITY);
        let (ack_tx_for_device, ack_rx_for_host) = channel(ACK_PACKET_CHANNEL_CAPACITY);

        // Create channel for communication between device and its internal interface.
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (completion_tx, completion_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        self.completion_rx = Some(completion_rx);

        task::spawn(Interface::new().run(
            ack_tx_for_device,
            req_rx_for_device,
            self.timestamp.clone(),
            self.memory.clone(),
            shutdown_rx,
            completion_tx,
        ));

        (req_tx_for_host, ack_rx_for_host)
    }

    pub(super) fn shutdown(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            // Signal shutdown to interface.
            drop(shutdown_tx);
            // Wait interface shutdown completion.
            let completion_rx = self.completion_rx.take().unwrap();
            task::block_on(completion_rx).ok();
        }

        self.shutdown_tx = None;
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[derive(Debug, Clone)]
pub(super) struct Timestamp(Arc<Mutex<time::Instant>>);

impl Timestamp {
    pub(super) fn new() -> Self {
        Self(Arc::new(Mutex::new(time::Instant::now())))
    }

    pub(super) async fn as_nanos(&self) -> u64 {
        let mut inner = self.0.lock().await;
        let ns: u64 = match inner.elapsed().as_nanos().try_into() {
            Ok(time) => time,
            Err(_) => {
                *inner = time::Instant::now();
                inner.elapsed().as_nanos() as u64
            }
        };
        ns
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::usb3::{
        protocol::{ack, command},
        register_map::abrm::*,
    };
    use async_std::future::timeout;
    use byteorder::{ReadBytesExt, WriteBytesExt, LE};

    const FAKE_LAYER_TO: Duration = Duration::from_millis(50);
    const GENCP_LAYER_TO: Duration = Duration::from_millis(500);

    use super::*;

    fn create_device() -> Device {
        let memory = Memory::new(Default::default());
        Device::new(memory)
    }

    #[test]
    fn test_device_run() {
        let mut device = create_device();
        device.run();
        device.shutdown();
    }

    #[test]
    fn test_handle_ctrl_packet() {
        let mut device = create_device();
        let (tx, rx) = device.run();

        // Write to TimestampLatch.
        let (tsl_addr, tsl_len, _) = TIMESTAMP_LATCH;
        let req_id = 0;
        let mut data = vec![];
        data.write_u32::<LE>(1).unwrap();
        let gencp_packet = command::WriteMem::new(tsl_addr, &data)
            .unwrap()
            .finalize(req_id);
        let mut buf = vec![];
        gencp_packet.serialize(&mut buf).unwrap();

        // Send ReadMem gencp packet over fake packet.
        let fake_req = FakeReqPacket::control(FakeReqKind::Send(buf));
        assert!(tx.try_send(fake_req).is_ok());
        let fake_ack = (task::block_on(timeout(FAKE_LAYER_TO, rx.recv())))
            .unwrap()
            .unwrap();
        assert_eq!(fake_ack.iface, IfaceKind::Control);
        assert_eq!(fake_ack.kind, FakeAckKind::SendAck);

        // Send recv request to obtain WriteMem gencp ack packet.
        // Device may be still in processing WriteMem packet, so we need to continue sending recv request
        // until ack packet is returned or GENCP_LAYER_TO is expired.
        let timer = Instant::now();
        let mut fake_ack;
        loop {
            let fake_req = FakeReqPacket::control(FakeReqKind::Recv);
            assert!(tx.try_send(fake_req).is_ok());
            fake_ack = (task::block_on(timeout(FAKE_LAYER_TO, rx.recv())))
                .unwrap()
                .unwrap();
            match fake_ack.kind {
                FakeAckKind::RecvAck(..) => break,
                _ => {}
            }
            if timer.elapsed() > GENCP_LAYER_TO {
                panic!();
            }
        }

        // Extract gencp ReadMem packet from fake ack packet.
        let gencp_writemem_ack = match fake_ack.kind {
            FakeAckKind::RecvAck(data) => data,
            _ => panic!(),
        };
        let ack_packet = ack::AckPacket::parse(&gencp_writemem_ack).unwrap();
        let ack_scd: ack::WriteMem = ack_packet.scd_as().unwrap();
        assert_eq!(ack_packet.request_id(), req_id);
        assert_eq!(ack_scd.length, tsl_len);

        // Get timestamp.
        let (ts_addr, ts_len, _) = TIMESTAMP;
        let req_id = 1;
        let gencp_packet = command::ReadMem::new(ts_addr, ts_len).finalize(req_id);
        let mut buf = vec![];
        gencp_packet.serialize(&mut buf).unwrap();

        // Send ReadMem gencp packet over fake packet.
        let fake_req = FakeReqPacket::control(FakeReqKind::Send(buf));
        assert!(tx.try_send(fake_req).is_ok());
        let fake_ack = (task::block_on(timeout(FAKE_LAYER_TO, rx.recv())))
            .unwrap()
            .unwrap();
        assert_eq!(fake_ack.kind, FakeAckKind::SendAck);
        assert_eq!(fake_ack.iface, IfaceKind::Control);

        // Send recv request to obtain ReadMem gencp ack packet.
        // Device may be still in processing ReadMem packet, so we need to continue sending recv request
        // while ack is returned or GENCP_LAYER_TO is expired.
        let timer = Instant::now();
        let mut fake_ack;
        loop {
            let fake_req = FakeReqPacket::control(FakeReqKind::Recv);
            assert!(tx.try_send(fake_req).is_ok());
            fake_ack = (task::block_on(timeout(FAKE_LAYER_TO, rx.recv())))
                .unwrap()
                .unwrap();
            match fake_ack.kind {
                FakeAckKind::RecvAck(..) => break,
                _ => {}
            }
            if timer.elapsed() > GENCP_LAYER_TO {
                panic!();
            }
        }

        // Extract gencp ReadMem packet from fake ack packet.
        let gencp_readmem_ack = match fake_ack.kind {
            FakeAckKind::RecvAck(data) => data,
            _ => panic!(),
        };

        let ack_packet = ack::AckPacket::parse(&gencp_readmem_ack).unwrap();
        assert_eq!(ack_packet.request_id(), req_id);
        let mut ack_scd: ack::ReadMem = ack_packet.scd_as().unwrap();

        // We wrote to TimestampLatch entry. So returned timestamp must be greater than zero.
        let timestamp = ack_scd.data.read_u64::<LE>().unwrap();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_handle_halt() {
        let mut device = create_device();
        let (tx, rx) = device.run();

        // Send set halt request.
        tx.try_send(FakeReqPacket::control(FakeReqKind::SetHalt))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::SetHaltAck);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Verify interface state is halted.
        tx.try_send(FakeReqPacket::control(FakeReqKind::Recv))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::IfaceHalted);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Send clear halt request.
        tx.try_send(FakeReqPacket::control(FakeReqKind::ClearHalt))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::ClearHaltAck);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Verify halted interface is cleared with meaningless gencp packet.
        tx.try_send(FakeReqPacket::control(FakeReqKind::Send(vec![1, 2, 3])))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::SendAck);
        assert_eq!(ack.iface, IfaceKind::Control);
    }

    #[test]
    fn test_handle_halt_while_processing() {
        let mut device = create_device();
        let (tx, rx) = device.run();

        // Send meaningless gencp packet.
        tx.try_send(FakeReqPacket::control(FakeReqKind::Send(vec![1, 2, 3])))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::SendAck);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Send set halt request.
        tx.try_send(FakeReqPacket::control(FakeReqKind::SetHalt))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::SetHaltAck);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Verify interface state is halted.
        tx.try_send(FakeReqPacket::control(FakeReqKind::Recv))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::IfaceHalted);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Send clear halt request.
        tx.try_send(FakeReqPacket::control(FakeReqKind::ClearHalt))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::ClearHaltAck);
        assert_eq!(ack.iface, IfaceKind::Control);

        // Verify halt state removes ack packets corresponding packets sent before halt.
        tx.try_send(FakeReqPacket::control(FakeReqKind::Recv))
            .unwrap();
        let ack = task::block_on(timeout(FAKE_LAYER_TO, rx.recv()))
            .unwrap()
            .unwrap();
        assert_eq!(ack.kind, FakeAckKind::RecvNak);
        assert_eq!(ack.iface, IfaceKind::Control);
    }
}
