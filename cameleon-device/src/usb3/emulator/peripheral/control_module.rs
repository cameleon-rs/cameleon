use std::sync::{atomic::AtomicBool, Arc};

use async_std::{
    prelude::*,
    sync::{channel, Mutex, Receiver, RwLock, Sender},
    task,
};
use futures::{channel::oneshot, select, FutureExt};

use super::{fake_protocol::IfaceKind, interface::IfaceState, memory::Memory, signal::*};

pub(super) struct ControlModule {
    req_rx: Receiver<Vec<u8>>,
    ctrl_manage_rx: Receiver<CtrlManagementSignal>,
    ack_tx: Sender<Vec<u8>>,
    memory: Arc<Mutex<Memory>>,
    iface_state: IfaceState,
    event_manage_tx: Sender<EventManagementSignal>,
    stream_manage_tx: Sender<StreamManagementSignal>,
}

impl ControlModule {
    pub(super) async fn run(mut self) {
        let mut req_rx = self.req_rx.fuse();
        let mut inner_event_rx = self.ctrl_manage_rx.fuse();
        let mut worker_manager = WorkerManager::new(
            self.memory.clone(),
            self.iface_state,
            self.event_manage_tx.clone(),
            self.stream_manage_tx.clone(),
        );

        loop {
            select! {
                req_data = req_rx.next().fuse() => {
                    if req_data.is_none() {
                        log::error!("main interface is unexpectedly stopped!");
                        break;
                    }
                    let req_data = req_data.unwrap();
                    let worker = worker_manager.worker();
                    task::spawn(worker.run(req_data));
                },

                event = inner_event_rx.next().fuse() => {
                    if event.is_none() {
                        log::error!("main interface is unexpectedly stopped!");
                        break;
                    }

                    let event = event.unwrap();
                    match event {
                        CtrlManagementSignal::Shutdown => {
                            break;
                        },

                        CtrlManagementSignal::SetHalt{iface, completed: _completed} => {
                            // We need to wait all workers completion even if the event is targeted at event/stream
                            // module to avoid race condition related to EI/SI register.
                            worker_manager.wait_completion().await;
                            match iface {
                                IfaceKind::Control => {}

                                IfaceKind::Event => {
                                    // TODO: Set zero to EI control register.
                                    let (completed_tx, completed_rx) = oneshot::channel();
                                    self.event_manage_tx.send(EventManagementSignal::Pause(completed_tx)).await;
                                    completed_rx.await.ok();
                                }

                                IfaceKind::Stream => {
                                    // TODO: Set zero to SI control register.
                                    let (completed_tx, completed_rx) = oneshot::channel();
                                    self.stream_manage_tx.send(StreamManagementSignal::Pause(completed_tx)).await;
                                    completed_rx.await.ok();
                                }
                            }
                        }
                    }
                }
            };
        }

        let (event_shutdown_tx, event_shutdown_rx) = oneshot::channel();
        let (stream_shutdown_tx, stream_shutdown_rx) = oneshot::channel();
        self.event_manage_tx
            .send(EventManagementSignal::Shutdown(event_shutdown_tx))
            .await;
        self.stream_manage_tx
            .send(StreamManagementSignal::Shutdown(stream_shutdown_tx))
            .await;

        event_shutdown_rx.await.ok();
        stream_shutdown_rx.await.ok();
    }
}

struct Worker {
    memory: Arc<Mutex<Memory>>,
    completed: Sender<()>,
    on_processing: Arc<AtomicBool>,
    iface_state: IfaceState,
    event_manage_tx: Sender<EventManagementSignal>,
    stream_manage_tx: Sender<StreamManagementSignal>,
}

impl Worker {
    async fn run(self, command: Vec<u8>) {
        todo!();
    }
}

struct WorkerManager {
    tx: Sender<()>,
    rx: Receiver<()>,
    memory: Arc<Mutex<Memory>>,
    iface_state: IfaceState,
    on_processing: Arc<AtomicBool>,
    event_manage_tx: Sender<EventManagementSignal>,
    stream_manage_tx: Sender<StreamManagementSignal>,
}

impl WorkerManager {
    fn new(
        memory: Arc<Mutex<Memory>>,
        iface_state: IfaceState,
        event_manage_tx: Sender<EventManagementSignal>,
        stream_manage_tx: Sender<StreamManagementSignal>,
    ) -> Self {
        let (tx, rx) = channel(1);
        let on_processing = Arc::new(AtomicBool::new(false));

        Self {
            tx,
            rx,
            memory,
            iface_state,
            on_processing,
            event_manage_tx,
            stream_manage_tx,
        }
    }

    fn worker(&self) -> Worker {
        Worker {
            memory: self.memory.clone(),
            completed: self.tx.clone(),
            on_processing: self.on_processing.clone(),
            iface_state: self.iface_state.clone(),
            event_manage_tx: self.event_manage_tx.clone(),
            stream_manage_tx: self.stream_manage_tx.clone(),
        }
    }

    async fn wait_completion(&mut self) {
        let (new_tx, new_rx) = channel(1);
        self.tx = new_tx;
        self.rx.recv().await.ok();
        self.rx = new_rx;
    }
}
