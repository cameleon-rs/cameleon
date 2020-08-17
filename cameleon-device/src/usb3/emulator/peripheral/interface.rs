use std::sync::Arc;

use async_std::{
    prelude::*,
    sync::{Receiver, RwLock, Sender, TryRecvError},
};
use futures::{channel::oneshot, select, FutureExt};

use super::{fake_protocol::*, signal::CtrlManagementSignal, EmulatorError, EmulatorResult};

pub(super) struct Interface {
    host_side_interface: (Sender<FakeAckPacket>, Receiver<FakeReqPacket>),

    ctrl_tx: CtrlEventSender,

    ack_rx: AckDataReceiver,
    /// Control
    iface_state: IfaceState,
}

impl Interface {
    pub(super) fn new(
        host_side_interface: (Sender<FakeAckPacket>, Receiver<FakeReqPacket>),
        ctrl_req_tx: Sender<Vec<u8>>,
        inner_ctrl_tx: Sender<CtrlManagementSignal>,
        ack_rx: AckDataReceiver,
    ) -> Self {
        Self {
            host_side_interface,
            ctrl_tx: CtrlEventSender {
                req_tx: ctrl_req_tx,
                inner_ctrl_tx,
            },
            ack_rx,
            iface_state: IfaceState::new(),
        }
    }

    pub(super) async fn run(
        mut self,
        shutdown: oneshot::Receiver<()>,
        _completed: oneshot::Sender<()>,
    ) {
        let mut receiver = self.host_side_interface.1.fuse();
        let mut shutdown = shutdown.fuse();
        let sender = self.host_side_interface.0;

        loop {
            let packet = select! {
                req_packet =  receiver.next().fuse() => {
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
                send_or_log(&sender, iface, FakeAckKind::ClearHaltAck);
                continue;
            } else if req_kind.is_set_halt() {
                if !self.iface_state.is_halt(iface).await {
                    self.iface_state
                        .set_state(iface, IfaceStateKind::Halt)
                        .await;

                    // Block until modules finish its processing correctly.
                    self.ctrl_tx.send_set_halt(iface).await;

                    // Discard all queued ack data.
                    while let Some(_) = self.ack_rx.try_recv(iface) {}
                }
                send_or_log(&sender, iface, FakeAckKind::SetHaltAck);
                continue;
            }

            // If corresponding interface is halted, ignore the request and send `FakeAckKind::IfaceHalted` back.
            if self.iface_state.is_halt(iface).await {
                send_or_log(&sender, iface, FakeAckKind::IfaceHalted);
                continue;
            };

            match (iface, req_kind) {
                (iface, FakeReqKind::Recv) => {
                    let data = self.ack_rx.try_recv(iface);
                    let ack_kind = match data {
                        Some(data) => FakeAckKind::RecvAck(data),
                        None => FakeAckKind::RecvNak,
                    };
                    send_or_log(&sender, iface, ack_kind);
                }

                (IfaceKind::Control, FakeReqKind::Send(data)) => {
                    let ack_kind = match self.ctrl_tx.send_req(data) {
                        Ok(()) => FakeAckKind::SendAck,
                        Err(err) => {
                            log::warn!("{}", err);
                            FakeAckKind::SendNak
                        }
                    };
                    send_or_log(&sender, iface, ack_kind);
                }

                (iface, req) => {
                    log::error!(
                        "invalid fake control packet. iface {:?}, req_kind: {:?}",
                        iface,
                        req
                    );
                    send_or_log(&sender, iface, FakeAckKind::BrokenReq);
                }
            };
        }

        self.ctrl_tx.send_shutdown().await;
        // Wait control module shutdown.
        // We delegate control of other modules to control module, so it's enough to just wait
        // control module shutdown.
        self.ack_rx.ctrl_buffer.recv().await;
        debug_assert!(match (
            self.ack_rx.event_buffer.try_recv(),
            self.ack_rx.stream_buffer.try_recv()
        ) {
            (Err(TryRecvError::Disconnected), Err(TryRecvError)) => true,
            _ => false,
        })
    }
}

pub(super) struct AckDataReceiver {
    ctrl_buffer: Receiver<Vec<u8>>,
    event_buffer: Receiver<Vec<u8>>,
    stream_buffer: Receiver<Vec<u8>>,
}

impl AckDataReceiver {
    fn try_recv(&self, iface: IfaceKind) -> Option<Vec<u8>> {
        match iface {
            IfaceKind::Control => self.ctrl_buffer.try_recv().ok(),
            IfaceKind::Event => self.event_buffer.try_recv().ok(),
            IfaceKind::Stream => self.stream_buffer.try_recv().ok(),
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

struct CtrlEventSender {
    /// This sender just propagate contents of `FakeReqPacket{iface: Control, req_kind:
    /// Send(data)}` to the control module.
    req_tx: Sender<Vec<u8>>,

    /// This sender handles requests that affect control mudule behavior itself.
    /// These requests must be processed if "normal" request channel is full. That's why we need
    /// this sender in addition to normal `req_tx` sender.
    inner_ctrl_tx: Sender<CtrlManagementSignal>,
}

impl CtrlEventSender {
    async fn send_set_halt(&self, iface: IfaceKind) {
        let (completed_tx, completed_rx) = oneshot::channel();

        let event = CtrlManagementSignal::SetHalt {
            iface,
            completed: completed_tx,
        };
        self.inner_ctrl_tx.send(event).await;
        completed_rx.await.ok();
    }

    async fn send_shutdown(&self) {
        let event = CtrlManagementSignal::Shutdown;
        self.inner_ctrl_tx.send(event).await;
    }

    fn send_req(&self, data: Vec<u8>) -> EmulatorResult<()> {
        self.req_tx
            .try_send(data)
            .map_err(|_| EmulatorError::FullBuffer)
    }
}

fn send_or_log(sender: &Sender<FakeAckPacket>, iface: IfaceKind, ack_kind: FakeAckKind) {
    let ack = FakeAckPacket::new(iface, ack_kind);
    match sender.try_send(ack) {
        Ok(()) => {}
        Err(_) => log::error!("can't send fake ack packet to the host"),
    }
}
