use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use async_std::{
    channel::{self, Receiver, Sender},
    prelude::*,
    sync::Mutex,
    task,
};

use cameleon_impl::memory::{prelude::*, MemoryError};

use super::{
    device::Timestamp,
    interface::IfaceState,
    memory::{Memory, SBRM},
    shared_queue::SharedQueue,
    signal::*,
    memory_event_handler::MemoryEventHandler,
    IfaceKind,
};

use super::control_protocol::{ack::AckSerialize, *};

pub(super) struct ControlModule {
    iface_state: IfaceState,
    memory: Arc<Mutex<Memory>>,
    timestamp: Timestamp,
    queue: SharedQueue<Vec<u8>>,
}


impl ControlModule {
    pub(super) fn new(
        iface_state: IfaceState,
        memory: Arc<Mutex<Memory>>,
        timestamp: Timestamp,
        queue: SharedQueue<Vec<u8>>,
    ) -> Self {
        Self {
            iface_state,
            memory,
            timestamp,
            queue,
        }
    }

    pub(super) async fn run(
        self,
        signal_tx: Sender<InterfaceSignal>,
        mut signal_rx: Receiver<ControlSignal>,
    ) {
        let event_handler = MemoryEventHandler::new(&mut *self.memory.lock().await).await;

        let mut worker_manager = WorkerManager::new(
            self.iface_state.clone(),
            self.memory.clone(),
            self.timestamp.clone(),
            event_handler,
            self.queue.clone(),
            signal_tx,
        )
        .await;

        while let Some(signal) = signal_rx.next().await {
            match signal {
                ControlSignal::ReceiveData(data) => {
                    let worker = worker_manager.worker();
                    task::spawn(worker.run(data));
                }

                ControlSignal::CancelJobs(_completed) => worker_manager.wait_completion().await,

                ControlSignal::ClearSiRegister => {
                    // TODO:
                    todo!()
                }

                ControlSignal::ClearEiRegister => {
                    // TODO:
                    todo!()
                }

                ControlSignal::Shutdown => {
                    worker_manager.wait_completion().await;
                    break;
                }
            }
        }
    }

}

struct WorkerManager {
    iface_state: IfaceState,
    memory: Arc<Mutex<Memory>>,
    timestamp: Timestamp,

    queue: SharedQueue<Vec<u8>>,
    signal_tx: Sender<InterfaceSignal>,

    on_processing: Arc<AtomicBool>,

    memory_event_handler: MemoryEventHandler,

    maximum_cmd_length: usize,
    maximum_ack_length: usize,

    /// Work as join handle coupled with `completed_rx`.
    /// All workers spawnd by the manager share this sender.
    completed_tx: Sender<()>,
    /// Work as join handle coupled with `completed_tx`.
    /// Manager can wait all spawned worker completion via this receiver.
    completed_rx: Receiver<()>,
}

impl WorkerManager {
    async fn new(
        iface_state: IfaceState,
        memory: Arc<Mutex<Memory>>,
        timestamp: Timestamp,
        memory_event_handler: MemoryEventHandler,
        queue: SharedQueue<Vec<u8>>,
        signal_tx: Sender<InterfaceSignal>,
    ) -> Self {
        let (completed_tx, completed_rx) = channel::bounded(1);
        let on_processing = Arc::new(AtomicBool::new(false));
        let (maximum_cmd_length, maximum_ack_length) = {
            let memory = memory.lock().await;
            (
                memory.read::<SBRM::MaximumCommandTransferLength>().unwrap() as usize,
                memory
                    .read::<SBRM::MaximumAcknowledgeTransferLength>()
                    .unwrap() as usize,
            )
        };

        Self {
            iface_state,
            memory,
            timestamp,

            queue,
            signal_tx,

            on_processing,

            memory_event_handler,

            maximum_cmd_length,
            maximum_ack_length,

            completed_tx,
            completed_rx,
        }
    }

    fn worker(&self) -> Worker {
        Worker {
            iface_state: self.iface_state.clone(),
            memory: self.memory.clone(),
            timestamp: self.timestamp.clone(),

            queue: self.queue.clone(),
            signal_tx: self.signal_tx.clone(),

            on_processing: self.on_processing.clone(),

            memory_event_handler: self.memory_event_handler.clone(),

            maximum_cmd_length: self.maximum_cmd_length,
            maximum_ack_length: self.maximum_ack_length,

            _completed: self.completed_tx.clone(),
        }
    }

    async fn wait_completion(&mut self) {
        let (new_tx, new_rx) = channel::bounded(1);
        // Drop old sender to wait workers completion only.
        self.completed_tx = new_tx;

        // Wait all workers completion.
        while self.completed_rx.next().await.is_some() {}

        self.completed_rx = new_rx;
    }
}

pub(super) struct Worker {
    iface_state: IfaceState,
    pub(super) memory: Arc<Mutex<Memory>>,
    pub(super) timestamp: Timestamp,

    queue: SharedQueue<Vec<u8>>,
    signal_tx: Sender<InterfaceSignal>,

    on_processing: Arc<AtomicBool>,

    memory_event_handler: MemoryEventHandler,

    maximum_cmd_length: usize,
    maximum_ack_length: usize,

    _completed: Sender<()>,
}

impl Worker {
    // TODO: Emulate pending situation.
    async fn run(self, command: Vec<u8>) {
        let cmd_packet = match self.try_parse_command(&command) {
            Some(packet) => packet,
            None => return,
        };
        let ccd = cmd_packet.ccd();

        // If sent command length is larger than SBRM::MaximumCommandTransferLength, return error.
        if (self.maximum_cmd_length) < command.len() {
            let ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidParameter, ccd.scd_kind())
                .finalize(ccd.request_id());
            self.enqueue_or_halt(ack);
            return;
        }

        // If another module is halted, return endpoint halted error
        if self.iface_state.is_halt(IfaceKind::Event).await {
            let ack =
                ack::ErrorAck::new(ack::UsbSpecificStatus::EventEndpointHalted, ccd.scd_kind())
                    .finalize(ccd.request_id());
            self.enqueue_or_halt(ack);
            return;
        } else if self.iface_state.is_halt(IfaceKind::Stream).await {
            let ack =
                ack::ErrorAck::new(ack::UsbSpecificStatus::StreamEndpointHalted, ccd.scd_kind())
                    .finalize(ccd.request_id());
            self.enqueue_or_halt(ack);
            return;
        }

        // If another thread is processing command simultaneously, return busy error ack.
        if self
            .on_processing
            .compare_and_swap(false, true, Ordering::Relaxed)
        {
            let ack = ack::ErrorAck::new(ack::GenCpStatus::Busy, ccd.scd_kind())
                .finalize(ccd.request_id());
            self.enqueue_or_halt(ack);
            return;
        }

        match ccd.scd_kind() {
            cmd::ScdKind::ReadMem => self.process_read_mem(cmd_packet).await,
            cmd::ScdKind::WriteMem => self.process_write_mem(cmd_packet).await,
            cmd::ScdKind::ReadMemStacked => self.process_read_mem_stacked(cmd_packet).await,
            cmd::ScdKind::WriteMemStacked => self.process_write_mem_stacked(cmd_packet).await,
        }

        self.on_processing.store(false, Ordering::Relaxed);
    }

    fn try_parse_command<'a>(&self, command: &'a [u8]) -> Option<cmd::CommandPacket<'a>> {
        match cmd::CommandPacket::parse(command) {
            Ok(packet) => Some(packet),
            Err(e) => {
                log::warn!("{}", e);

                // Can't parse even CCD, so return error ack packet with dummy scd kind and dummy request id.
                let ack =
                    ack::ErrorAck::new(ack::GenCpStatus::InvalidParameter, ack::ScdKind::ReadMem)
                        .finalize(0);
                self.enqueue_or_halt(ack);

                None
            }
        }
    }

    async fn process_read_mem(&self, command: cmd::CommandPacket<'_>) {
        let scd: cmd::ReadMem = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id();
        let scd_kind = ccd.scd_kind();

        let memory = self.memory.lock().await;
        let address = scd.address as usize;
        let read_length = scd.read_length as usize;

        match memory.read_raw(address..address + read_length) {
            Ok(data) => {
                let ack = ack::ReadMem::new(data).finalize(req_id);
                self.enqueue_or_halt(ack);
            }

            Err(MemoryError::InvalidAddress) => {
                let ack =
                    ack::ErrorAck::new(ack::GenCpStatus::InvalidAddress, scd_kind).finalize(req_id);
                self.enqueue_or_halt(ack);
            }

            Err(MemoryError::AddressNotReadable) => {
                let ack =
                    ack::ErrorAck::new(ack::GenCpStatus::AccessDenied, scd_kind).finalize(req_id);
                self.enqueue_or_halt(ack);
            }

            Err(MemoryError::AddressNotWritable) | Err(MemoryError::InvalidRegisterData(..)) => {
                unreachable!()
            }
        };
    }

    async fn process_write_mem(&self, command: cmd::CommandPacket<'_>) {
        let scd: cmd::WriteMem = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id();
        let scd_kind = ccd.scd_kind();

        let mut memory = self.memory.lock().await;
        match memory.write_raw(scd.address as usize, scd.data) {
            Ok(()) => {
                // Explicitly drop memory to avoid race condition.
                drop(memory);

                let error_ack = self.memory_event_handler.handle_events(self, scd_kind).await;
                if let Some(error_ack) = error_ack {
                    self.enqueue_or_halt(error_ack.finalize(req_id));
                } else {
                    let ack = ack::WriteMem::new(scd.data.len() as u16).finalize(req_id);
                    self.enqueue_or_halt(ack);

                }
            }

            Err(MemoryError::InvalidAddress) => {
                let ack =
                    ack::ErrorAck::new(ack::GenCpStatus::InvalidAddress, scd_kind).finalize(req_id);
                self.enqueue_or_halt(ack);
            }

            Err(MemoryError::AddressNotWritable) => {
                let ack =
                    ack::ErrorAck::new(ack::GenCpStatus::WriteProtect, scd_kind).finalize(req_id);
                self.enqueue_or_halt(ack);
            }

            Err(MemoryError::AddressNotReadable) | Err(MemoryError::InvalidRegisterData(..)) => {
                unreachable!()
            }
        };
    }

    async fn process_read_mem_stacked(&self, command: cmd::CommandPacket<'_>) {
        let _scd: cmd::WriteMemStacked = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id();
        let scd_kind = ccd.scd_kind();

        // TODO: Should we implement this command?
        let ack = ack::ErrorAck::new(ack::GenCpStatus::NotImplemented, scd_kind).finalize(req_id);
        self.enqueue_or_halt(ack);
    }

    async fn process_write_mem_stacked(&self, command: cmd::CommandPacket<'_>) {
        let _scd: cmd::WriteMemStacked = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id();
        let scd_kind = ccd.scd_kind();

        // TODO: Should we implement this command?
        let ack = ack::ErrorAck::new(ack::GenCpStatus::NotImplemented, scd_kind).finalize(req_id);
        self.enqueue_or_halt(ack);
    }

    fn try_extract_scd<'a, T>(&self, command: &cmd::CommandPacket<'a>) -> Option<T>
    where
        T: cmd::ParseScd<'a>,
    {
        match command.scd_as::<T>() {
            Ok(scd) => Some(scd),
            Err(_) => {
                let ccd = command.ccd();
                let ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidParameter, ccd.scd_kind())
                    .finalize(ccd.request_id());
                self.enqueue_or_halt(ack);
                None
            }
        }
    }

    fn enqueue_or_halt<T>(&self, ack: ack::AckPacket<T>)
    where
        T: AckSerialize,
    {
        let mut buf = vec![];

        if let Err(e) = ack.serialize(&mut buf) {
            log::error!("{}", e);
            return;
        }

        // If ack packet length is larger than maximu_ack_length, return error.
        let buf = if (self.maximum_ack_length) < buf.len() {
            let err_ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidParameter, ack.ccd.scd_kind)
                .finalize(ack.ccd.request_id);
            let mut buf = vec![];
            err_ack.serialize(&mut buf).unwrap();
            buf
        } else {
            buf
        };

        if !self.queue.enqueue(buf) {
            log::warn!("control queue is full, entering a halted state");
            self.try_send_signal(InterfaceSignal::Halt(IfaceKind::Control));
        }
    }

    pub(super) fn try_send_signal(&self, signal: impl Into<InterfaceSignal>) {
        match self.signal_tx.try_send(signal.into()) {
            Ok(()) => {}
            Err(_) => {
                log::error!("Control module -> Interface channel is full");
            }
        }
    }
}

