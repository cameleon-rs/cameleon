use std::sync::Arc;

use async_std::{
    channel::{self, Receiver, Sender, TrySendError},
    prelude::*,
    sync::{Mutex, RwLock},
    task,
};
use futures::{channel::oneshot, select, FutureExt};

use super::{
    control_module::ControlModule, device::Timestamp, event_module::EventModule, fake_protocol::*,
    memory::Memory, shared_queue::SharedQueue, signal::*, stream_module::StreamModule,
};

pub(super) struct Interface {
    iface_state: IfaceState,
    memory: Arc<Mutex<Memory>>,
    timestamp: Timestamp,

    ctrl_queue: SharedQueue<Vec<u8>>,
    event_queue: SharedQueue<Vec<u8>>,
    stream_queue: SharedQueue<Vec<u8>>,
}

const SHARED_QUEUE_SIZE: usize = 32;
const CHANNEL_CAPACITY: usize = 128;

impl Interface {
    pub(super) fn new(memory: Arc<Mutex<Memory>>, timestamp: Timestamp) -> Self {
        Self {
            iface_state: IfaceState::new(),
            memory,
            timestamp,

            ctrl_queue: SharedQueue::new(SHARED_QUEUE_SIZE),
            event_queue: SharedQueue::new(SHARED_QUEUE_SIZE),
            stream_queue: SharedQueue::new(SHARED_QUEUE_SIZE),
        }
    }

    pub(super) async fn run(
        self,
        fake_ack_tx: Sender<FakeAckPacket>,
        fake_req_rx: Receiver<FakeReqPacket>,
        shutdown: oneshot::Receiver<()>,
        _completed: oneshot::Sender<()>,
    ) {
        // Construct interface signal channel.
        let (iface_signal_tx, signal_rx) = channel::bounded(CHANNEL_CAPACITY);

        // Construct and spawn control module.
        let ctrl_signal_tx = self.spawn_ctrl_module(iface_signal_tx.clone());
        // Construct and spawn event module.
        let event_signal_tx = self.spawn_event_module(iface_signal_tx.clone());
        // Construct and spawn stream module.
        let stream_signal_tx = self.spawn_stream_module(iface_signal_tx);

        let signal_tx = ModuleSignalTx::new(ctrl_signal_tx, event_signal_tx, stream_signal_tx);

        let mut signal_rx = signal_rx.fuse();
        let mut fake_req_rx = fake_req_rx.fuse();
        let mut shutdown = shutdown.fuse();

        loop {
            select! {
                packet = fake_req_rx.next().fuse() => {
                    match packet {
                        Some(packet) => {
                            self.handle_packet(packet, &fake_ack_tx, &signal_tx).await
                        }
                        None => {
                            log::error!("host side sender is dropped");
                            break
                        }
                    }
                },

                signal = signal_rx.next().fuse() => {
                    match signal {
                        Some(signal) => {
                            self.handle_signal(signal, &signal_tx).await
                        }
                        None => {
                            log::error!("all modules are dropped");
                            break
                        }
                    }
                }

                _ = shutdown => {
                    break;
                }
            }
        }

        // Send shutdown signal to each module.
        signal_tx.send_ctrl(ControlSignal::Shutdown);
        signal_tx.send_event(EventSignal::Shutdown);
        signal_tx.send_stream(StreamSignal::Shutdown);

        // Wait shutdown.
        while signal_rx.next().await.is_some() {}
    }

    fn spawn_ctrl_module(&self, signal_tx: Sender<InterfaceSignal>) -> Sender<ControlSignal> {
        let (ctrl_signal_tx, ctrl_signal_rx) = channel::bounded(CHANNEL_CAPACITY);

        // Construct and spawn control module.
        let control_module = ControlModule::new(
            self.iface_state.clone(),
            self.memory.clone(),
            self.timestamp.clone(),
            self.ctrl_queue.clone(),
        );
        task::spawn(control_module.run(signal_tx, ctrl_signal_rx));

        ctrl_signal_tx
    }

    fn spawn_event_module(&self, signal_tx: Sender<InterfaceSignal>) -> Sender<EventSignal> {
        let (event_signal_tx, event_signal_rx) = channel::bounded(CHANNEL_CAPACITY);

        // Construct and spawn control module.
        let event_module = EventModule::new(self.event_queue.clone(), 0);
        task::spawn(event_module.run(signal_tx, event_signal_rx));

        event_signal_tx
    }

    fn spawn_stream_module(&self, signal_tx: Sender<InterfaceSignal>) -> Sender<StreamSignal> {
        let (stream_signal_tx, stream_signal_rx) = channel::bounded(CHANNEL_CAPACITY);

        // Construct and spawn control module.
        let stream_module = StreamModule::new(self.timestamp.clone(), self.stream_queue.clone());
        task::spawn(stream_module.run(signal_tx, stream_signal_rx));

        stream_signal_tx
    }

    /// Handle a fake request packet sent from a host.
    async fn handle_packet(
        &self,
        packet: FakeReqPacket,
        ack_tx: &Sender<FakeAckPacket>,
        signal_tx: &ModuleSignalTx,
    ) {
        let iface = packet.iface;
        let req_kind = packet.kind;

        // Handle claer halt request.
        if req_kind.is_clear_halt() {
            self.iface_state
                .set_state(iface, IfaceStateKind::Ready)
                .await;
            send_ack(&ack_tx, iface, FakeAckKind::ClearHaltAck);
            return;
        }

        // Handle set halt request.
        if req_kind.is_set_halt() {
            self.set_halt(iface, signal_tx).await;
            send_ack(&ack_tx, iface, FakeAckKind::SetHaltAck);
            return;
        }

        // If corresponding interface is halted, ignore the request and send `FakeAckKind::IfaceHalted` back.
        if self.iface_state.is_halt(iface).await {
            send_ack(ack_tx, iface, FakeAckKind::IfaceHalted);
            return;
        };

        // Handle request.
        match (iface, req_kind) {
            (iface, FakeReqKind::Recv) => {
                let data = match iface {
                    IfaceKind::Control => self.ctrl_queue.dequeue(),
                    IfaceKind::Event => self.event_queue.dequeue(),
                    IfaceKind::Stream => self.stream_queue.dequeue(),
                };

                let ack_kind = match data {
                    Some(data) => FakeAckKind::RecvAck(data),
                    None => FakeAckKind::RecvNak,
                };
                send_ack(&ack_tx, iface, ack_kind);
            }

            (IfaceKind::Control, FakeReqKind::Send(data)) => {
                signal_tx.send_ctrl(ControlSignal::ReceiveData(data));
                send_ack(&ack_tx, iface, FakeAckKind::SendAck);
            }

            (iface, req) => {
                log::error!(
                    "invalid fake control packet. iface {:?}, req_kind: {:?}",
                    iface,
                    req
                );
                send_ack(&ack_tx, iface, FakeAckKind::BrokenReq);
            }
        };
    }

    /// Handle a interface signal sent from modules.
    async fn handle_signal(&self, signal: InterfaceSignal, signal_tx: &ModuleSignalTx) {
        match signal {
            InterfaceSignal::_ToControl(signal) => signal_tx.send_ctrl(signal),
            InterfaceSignal::ToEvent(signal) => signal_tx.send_event(signal),
            InterfaceSignal::_ToStream(signal) => signal_tx.send_stream(signal),
            InterfaceSignal::Halt(iface) => self.set_halt(iface, signal_tx).await,
        }
    }

    async fn set_halt(&self, iface: IfaceKind, signal_tx: &ModuleSignalTx) {
        if self.iface_state.is_halt(iface).await {
            return;
        }

        // Cancel all jobs running on cotrol module.
        let (completed_tx, completed_rx) = oneshot::channel();
        signal_tx.send_ctrl(ControlSignal::CancelJobs(completed_tx));
        completed_rx.await.ok();

        // Disable module.
        self.disable_module(iface, signal_tx).await;

        // Clear queue.
        match iface {
            IfaceKind::Control => self.ctrl_queue.clear(),
            IfaceKind::Event => self.event_queue.clear(),
            IfaceKind::Stream => self.stream_queue.clear(),
        }

        // Set iface state as halt.
        self.iface_state
            .set_state(iface, IfaceStateKind::Halt)
            .await;
    }

    // Disable module. No need to disable when iface == IfaceKind::Control.
    async fn disable_module(&self, iface: IfaceKind, signal_tx: &ModuleSignalTx) {
        if iface == IfaceKind::Event {
            signal_tx.send_ctrl(ControlSignal::ClearEiRegister);
            let (completed_tx, completed_rx) = oneshot::channel();
            signal_tx.send_event(EventSignal::Disable(completed_tx));
            completed_rx.await.ok();
        } else if iface == IfaceKind::Stream {
            signal_tx.send_ctrl(ControlSignal::ClearSiRegister);
            let (completed_tx, completed_rx) = oneshot::channel();
            signal_tx.send_stream(StreamSignal::Disable(completed_tx));
            completed_rx.await.ok();
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

    async fn set_state(&self, iface: IfaceKind, state: IfaceStateKind) {
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

struct ModuleSignalTx {
    ctrl: Sender<ControlSignal>,
    event: Sender<EventSignal>,
    stream: Sender<StreamSignal>,
}

impl ModuleSignalTx {
    fn new(
        ctrl: Sender<ControlSignal>,
        event: Sender<EventSignal>,
        stream: Sender<StreamSignal>,
    ) -> Self {
        Self {
            ctrl,
            event,
            stream,
        }
    }

    fn send_ctrl(&self, signal: ControlSignal) {
        Self::send(&self.ctrl, signal)
    }

    fn send_event(&self, signal: EventSignal) {
        Self::send(&self.event, signal)
    }

    fn send_stream(&self, signal: StreamSignal) {
        Self::send(&self.stream, signal)
    }

    fn send<T>(sender: &Sender<T>, signal: T)
    where
        T: Send,
    {
        match sender.try_send(signal) {
            Ok(()) => {}
            Err(err) => {
                // TODO: Handle condition where channel is full.
                panic!("{}", err);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IfaceStateKind {
    Halt,
    Ready,
}

fn send_ack(sender: &Sender<FakeAckPacket>, iface: IfaceKind, ack_kind: FakeAckKind) {
    let ack = FakeAckPacket::new(iface, ack_kind);
    match sender.try_send(ack) {
        Ok(()) => {}
        Err(TrySendError::Closed(..)) => {
            log::error!("can't send fake ack packet to the host. cause: recv end is disconnected.")
        }
        Err(TrySendError::Full(..)) => {
            log::error!("can't send fake ack packet to the host. cause: recv end is full.")
        }
    }
}
