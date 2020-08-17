use async_std::{
    prelude::*,
    sync::{Receiver, Sender},
    task,
};
use futures::{channel::oneshot, select, FutureExt};

use super::{fake_protocol::*, EmulatorError, EmulatorResult};

pub(super) struct Interface {
    host_side_interface: (Sender<FakeAckPacket>, Receiver<FakeReqPacket>),

    ctrl_sender: CtrlEventSender,

    ack_rx: AckDataReceiver,
    iface_state: IfaceState,
}

impl Interface {
    pub(super) fn new(
        host_side_interface: (Sender<FakeAckPacket>, Receiver<FakeReqPacket>),
        ctrl_req_tx: Sender<Vec<u8>>,
        inner_ctrl_tx: Sender<InnerCtrlEvent>,
        ack_rx: AckDataReceiver,
    ) -> Self {
        Self {
            host_side_interface,
            ctrl_sender: CtrlEventSender {
                req_tx: ctrl_req_tx,
                inner_ctrl_tx,
            },
            ack_rx,
            iface_state: IfaceState::new(),
        }
    }

    pub(super) async fn run(mut self, shutdown: oneshot::Receiver<()>) {
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
                self.iface_state.set_state(iface, IfaceStateKind::Ready);
                send_or_log(&sender, iface, FakeAckKind::ClearHaltAck);
                continue;
            } else if req_kind.is_set_halt() {
                if !self.iface_state.is_halt(iface) {
                    self.iface_state.set_state(iface, IfaceStateKind::Halt);
                    // Block until modules finish its processing correctly.
                    task::block_on(self.ctrl_sender.send_set_halt(iface));
                    // Discard all queued ack data.
                    while let Some(_) = self.ack_rx.try_recv(iface) {}
                }
                send_or_log(&sender, iface, FakeAckKind::SetHaltAck);
                continue;
            }

            // If corresponding interface is halted, ignore the request and send `FakeAckKind::IfaceHalted` back.
            if self.iface_state.is_halt(iface) {
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
                    let ack_kind = match self.ctrl_sender.send_req(data) {
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

        task::block_on(self.ctrl_sender.send_shutdown());
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

pub(super) enum InnerCtrlEvent {
    SetHalt {
        iface: IfaceKind,
        completed: oneshot::Sender<()>,
    },

    ShutDown {
        completed: oneshot::Sender<()>,
    },
}

struct CtrlEventSender {
    /// This sender just propagate contents of `FakeReqPacket{iface: Control, req_kind:
    /// Send(data)}` to the control module.
    req_tx: Sender<Vec<u8>>,

    /// This sender handles requests that affect control mudule behavior itself.
    /// These requests must be processed if "normal" request channel is full. That's why we need
    /// this sender in addition to normal `req_tx` sender.
    inner_ctrl_tx: Sender<InnerCtrlEvent>,
}

impl CtrlEventSender {
    async fn send_set_halt(&self, iface: IfaceKind) {
        let (completed_tx, completed_rx) = oneshot::channel();

        let event = InnerCtrlEvent::SetHalt {
            iface,
            completed: completed_tx,
        };
        self.inner_ctrl_tx.send(event).await;
        completed_rx.await.ok();
    }

    async fn send_shutdown(&self) {
        let (completed_tx, completed_rx) = oneshot::channel();
        let event = InnerCtrlEvent::ShutDown {
            completed: completed_tx,
        };
        self.inner_ctrl_tx.send(event).await;
        completed_rx.await.ok();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IfaceStateKind {
    Halt,
    Ready,
}

struct IfaceState {
    ctrl_state: IfaceStateKind,
    event_state: IfaceStateKind,
    stream_state: IfaceStateKind,
}

impl IfaceState {
    fn new() -> Self {
        use IfaceStateKind::*;
        Self {
            ctrl_state: Ready,
            event_state: Ready,
            stream_state: Ready,
        }
    }

    fn state(&self, iface: IfaceKind) -> IfaceStateKind {
        match iface {
            IfaceKind::Control => self.ctrl_state,
            IfaceKind::Event => self.event_state,
            IfaceKind::Stream => self.stream_state,
        }
    }

    fn set_state(&mut self, iface: IfaceKind, state: IfaceStateKind) {
        match iface {
            IfaceKind::Control => self.ctrl_state = state,
            IfaceKind::Event => self.event_state = state,
            IfaceKind::Stream => self.stream_state = state,
        }
    }

    fn is_halt(&self, iface: IfaceKind) -> bool {
        match self.state(iface) {
            IfaceStateKind::Halt => true,
            _ => false,
        }
    }
}
