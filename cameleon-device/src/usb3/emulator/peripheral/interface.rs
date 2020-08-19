use std::sync::Arc;

use async_std::{
    prelude::*,
    sync::{channel, Mutex, Receiver, RwLock, Sender, TrySendError},
    task,
};
use futures::{channel::oneshot, select, FutureExt};

use super::{
    control_module::ControlModule, device::Timestamp, fake_protocol::*, memory::Memory,
    signal::CtrlSignal,
};

pub(super) struct Interface {
    iface_state: IfaceState,
}

const CTRL_CHANNEL_CAPACITY: usize = 32;
const CTRL_ACK_DATA_CHANNELE_CAPACITY: usize = 32;
const EVENT_ACK_DATA_CHANNELE_CAPACITY: usize = 32;
const STREAM_ACK_DATA_CHANNEL_CAPACITY: usize = 32;

impl Interface {
    pub(super) fn new() -> Self {
        Self {
            iface_state: IfaceState::new(),
        }
    }

    fn spawn_ctrl_module(
        &self,
        timestamp: Timestamp,
        memory: Arc<Mutex<Memory>>,
    ) -> (Sender<CtrlSignal>, AckDataReceiver) {
        // Create channel to manage control module.
        let (ctrl_tx, ctrl_rx) = channel(CTRL_CHANNEL_CAPACITY);

        // Create channel to recieve data from control/event/stream module.
        let (ack_tx, ack_rx) = ack_data_channel(
            CTRL_ACK_DATA_CHANNELE_CAPACITY,
            EVENT_ACK_DATA_CHANNELE_CAPACITY,
            STREAM_ACK_DATA_CHANNEL_CAPACITY,
        );

        // Contruct and spawn control module.
        let control_module = ControlModule::new(memory, timestamp);
        task::spawn(control_module.run(self.iface_state.clone(), ctrl_rx, ack_tx));

        (ctrl_tx, ack_rx)
    }

    pub(super) async fn run(
        mut self,
        tx_for_host: Sender<FakeAckPacket>,
        rx_for_host: Receiver<FakeReqPacket>,
        timestamp: Timestamp,
        memory: Arc<Mutex<Memory>>,
        shutdown: oneshot::Receiver<()>,
        _completed: oneshot::Sender<()>,
    ) {
        // Contruct and spawn control module. We delegate other module contruction to the control module, so no need to build them.
        let (ctrl_tx_for_mod, ack_rx_for_mod) = self.spawn_ctrl_module(timestamp, memory);
        let mut rx_for_host = rx_for_host.fuse();
        let mut shutdown = shutdown.fuse();

        loop {
            let packet = select! {
                req_packet =  rx_for_host.next().fuse() => {
                    match req_packet {
                        Some(packet) => {
                            packet
                        }
                        None => {
                            log::error!("host side sender is dropped");
                            break
                        }
                    }
                },

                _ = shutdown => {
                    break;
                }
            };

            let iface = packet.iface;
            let req_kind = packet.req_kind;

            // Handle request related to halt.
            if req_kind.is_clear_halt() {
                self.iface_state
                    .set_state(iface, IfaceStateKind::Ready)
                    .await;
                send_or_log(&tx_for_host, iface, FakeAckKind::ClearHaltAck);
                continue;
            } else if req_kind.is_set_halt() {
                if !self.iface_state.is_halt(iface).await {
                    self.iface_state
                        .set_state(iface, IfaceStateKind::Halt)
                        .await;

                    // Block until modules finish its processing correctly.
                    let (completed_tx, completed_rx) = oneshot::channel();
                    ctrl_tx_for_mod.send(CtrlSignal::SetHalt {
                        iface,
                        completed: completed_tx,
                    });
                    completed_rx.await.ok();

                    // Discard all queued ack data.
                    while let Some(_) = ack_rx_for_mod.try_recv(iface) {}
                }
                send_or_log(&tx_for_host, iface, FakeAckKind::SetHaltAck);
                continue;
            }

            // If corresponding interface is halted, ignore the request and send `FakeAckKind::IfaceHalted` back.
            if self.iface_state.is_halt(iface).await {
                send_or_log(&tx_for_host, iface, FakeAckKind::IfaceHalted);
                continue;
            };

            match (iface, req_kind) {
                (iface, FakeReqKind::Recv) => {
                    let data = ack_rx_for_mod.try_recv(iface);
                    let ack_kind = match data {
                        Some(data) => FakeAckKind::RecvAck(data),
                        None => FakeAckKind::RecvNak,
                    };
                    send_or_log(&tx_for_host, iface, ack_kind);
                }

                (IfaceKind::Control, FakeReqKind::Send(data)) => {
                    let ack_kind = match ctrl_tx_for_mod.try_send(CtrlSignal::SendDataReq(data)) {
                        Ok(()) => FakeAckKind::SendAck,
                        Err(err) => {
                            log::warn!("{}", err);
                            FakeAckKind::SendNak
                        }
                    };
                    send_or_log(&tx_for_host, iface, ack_kind);
                }

                (iface, req) => {
                    log::error!(
                        "invalid fake control packet. iface {:?}, req_kind: {:?}",
                        iface,
                        req
                    );
                    send_or_log(&tx_for_host, iface, FakeAckKind::BrokenReq);
                }
            };
        }

        // Send shutdown signal to control module.
        // We delegate control of other modules to control module, so it's enough to just wait
        // control module shutdown.
        let (completed_tx, completed_rx) = oneshot::channel();
        ctrl_tx_for_mod
            .send(CtrlSignal::Shutdown(completed_tx))
            .await;
        completed_rx.await.ok();

        debug_assert!(match (
            ack_rx_for_mod.try_recv(IfaceKind::Control),
            ack_rx_for_mod.try_recv(IfaceKind::Event),
            ack_rx_for_mod.try_recv(IfaceKind::Stream)
        ) {
            (None, None, None) => true,
            _ => false,
        })
    }
}

pub(super) fn ack_data_channel(
    ctrl_cap: usize,
    event_cap: usize,
    stream_cap: usize,
) -> (AckDataSender, AckDataReceiver) {
    let (ctrl_tx, ctrl_rx) = channel(ctrl_cap);
    let (event_tx, event_rx) = channel(event_cap);
    let (stream_tx, stream_rx) = channel(stream_cap);
    (
        AckDataSender {
            ctrl: ctrl_tx,
            event: event_tx,
            stream: stream_tx,
        },
        AckDataReceiver {
            ctrl: ctrl_rx,
            event: event_rx,
            stream: stream_rx,
        },
    )
}

pub(super) struct AckDataSender {
    pub(super) ctrl: Sender<Vec<u8>>,
    pub(super) event: Sender<Vec<u8>>,
    pub(super) stream: Sender<Vec<u8>>,
}

pub(super) struct AckDataReceiver {
    ctrl: Receiver<Vec<u8>>,
    event: Receiver<Vec<u8>>,
    stream: Receiver<Vec<u8>>,
}

impl AckDataReceiver {
    fn try_recv(&self, iface: IfaceKind) -> Option<Vec<u8>> {
        match iface {
            IfaceKind::Control => self.ctrl.try_recv().ok(),
            IfaceKind::Event => self.event.try_recv().ok(),
            IfaceKind::Stream => self.stream.try_recv().ok(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct IfaceState {
    ctrl_state: Arc<RwLock<IfaceStateKind>>,
    event_state: Arc<RwLock<IfaceStateKind>>,
    stream_state: Arc<RwLock<IfaceStateKind>>,
}

impl IfaceState {
    fn new() -> Self {
        use IfaceStateKind::*;
        Self {
            ctrl_state: Arc::new(RwLock::new(Ready)),
            event_state: Arc::new(RwLock::new(Ready)),
            stream_state: Arc::new(RwLock::new(Ready)),
        }
    }

    async fn set_state(&mut self, iface: IfaceKind, state: IfaceStateKind) {
        match iface {
            IfaceKind::Control => *self.ctrl_state.write().await = state,
            IfaceKind::Event => *self.event_state.write().await = state,
            IfaceKind::Stream => *self.stream_state.write().await = state,
        }
    }

    pub(super) async fn is_halt(&self, iface: IfaceKind) -> bool {
        let state = match iface {
            IfaceKind::Control => self.ctrl_state.read().await,
            IfaceKind::Event => self.event_state.read().await,
            IfaceKind::Stream => self.stream_state.read().await,
        };
        *state == IfaceStateKind::Halt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IfaceStateKind {
    Halt,
    Ready,
}

fn send_or_log(sender: &Sender<FakeAckPacket>, iface: IfaceKind, ack_kind: FakeAckKind) {
    let ack = FakeAckPacket::new(iface, ack_kind);
    match sender.try_send(ack) {
        Ok(()) => {}
        Err(TrySendError::Disconnected(..)) => {
            log::error!("can't send fake ack packet to the host. cause: recv end is disconnected.")
        }
        Err(TrySendError::Full(..)) => {
            log::error!("can't send fake ack packet to the host. cause: recv end is full.")
        }
    }
}
