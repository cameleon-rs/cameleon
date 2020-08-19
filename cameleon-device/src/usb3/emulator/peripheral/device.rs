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
    tx_for_host: Option<Sender<FakeReqPacket>>,
    rx_for_host: Option<Receiver<FakeAckPacket>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    completion_rx: Option<oneshot::Receiver<()>>,
}

impl Device {
    pub(super) fn new(memory: Memory) -> Self {
        Self {
            timestamp: Timestamp::new(),
            memory: Arc::new(Mutex::new(memory)),
            tx_for_host: None,
            rx_for_host: None,
            shutdown_tx: None,
            completion_rx: None,
        }
    }

    pub(super) fn connect(&mut self) -> Option<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)> {
        if let (Some(tx), Some(rx)) = (self.tx_for_host.take(), self.rx_for_host.take()) {
            Some((tx, rx))
        } else {
            None
        }
    }

    pub(super) fn run(&mut self) {
        // Create channels for communication between device and host.
        let (req_tx_for_host, req_rx_for_device) = channel(REQ_PACKET_CHANNEL_CAPACITY);
        let (ack_tx_for_device, ack_rx_for_host) = channel(ACK_PACKET_CHANNEL_CAPACITY);
        self.tx_for_host = Some(req_tx_for_host);
        self.rx_for_host = Some(ack_rx_for_host);

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
        self.tx_for_host = None;
        self.rx_for_host = None;
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
