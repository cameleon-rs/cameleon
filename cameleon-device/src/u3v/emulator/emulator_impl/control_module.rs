use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_std::{
    prelude::*,
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use thiserror::Error;

use cameleon_impl::memory::{prelude::*, MemoryError, MemoryObserver};

use super::{
    device::Timestamp,
    interface::IfaceState,
    memory::{Memory, ABRM, SBRM},
    shared_queue::SharedQueue,
    signal::*,
    IfaceKind,
};

use ack::AckSerialize;

pub(super) struct ControlModule {
    iface_state: IfaceState,
    memory: Arc<Mutex<Memory>>,
    timestamp: Timestamp,
    queue: SharedQueue<Vec<u8>>,
}

const MEMORY_EVENT_CHANNEL_CAPACITY: usize = 100;

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
        let event_handler = self.register_observers().await;

        let (maximum_cmd_length, maximum_ack_length) = {
            let memory = self.memory.lock().await;
            (
                memory.read::<SBRM::MaximumCommandTransferLength>().unwrap() as usize,
                memory
                    .read::<SBRM::MaximumAcknowledgeTransferLength>()
                    .unwrap() as usize,
            )
        };

        let mut worker_manager = WorkerManager::new(
            self.iface_state.clone(),
            self.memory.clone(),
            self.timestamp.clone(),
            event_handler,
            self.queue.clone(),
            signal_tx,
            maximum_cmd_length,
            maximum_ack_length,
        );

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

    async fn register_observers(&self) -> MemoryEventHandler {
        let mut memory = self.memory.lock().await;
        let (tx, rx) = channel(MEMORY_EVENT_CHANNEL_CAPACITY);
        memory.register_observer::<ABRM::TimestampLatch, _>(TimestampLatchObserver { sender: tx });

        MemoryEventHandler { rx }
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
    fn new(
        iface_state: IfaceState,
        memory: Arc<Mutex<Memory>>,
        timestamp: Timestamp,
        memory_event_handler: MemoryEventHandler,
        queue: SharedQueue<Vec<u8>>,
        signal_tx: Sender<InterfaceSignal>,
        maximum_cmd_length: usize,
        maximum_ack_length: usize,
    ) -> Self {
        let (completed_tx, completed_rx) = channel(1);
        let on_processing = Arc::new(AtomicBool::new(false));

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
        let (new_tx, new_rx) = channel(1);
        // Drop old sender to wait workers completion only.
        self.completed_tx = new_tx;

        // Wait all workers completion.
        while self.completed_rx.next().await.is_some() {}

        self.completed_rx = new_rx;
    }
}

struct Worker {
    iface_state: IfaceState,
    memory: Arc<Mutex<Memory>>,
    timestamp: Timestamp,

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

                self.memory_event_handler.handle_events(self).await;
                let ack = ack::WriteMem::new(scd.data.len() as u16).finalize(req_id);
                self.enqueue_or_halt(ack);
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

    fn try_send_signal(&self, signal: InterfaceSignal) {
        match self.signal_tx.try_send(signal) {
            Ok(()) => {}
            Err(_) => {
                log::error!("Control module -> Interface channel is full");
            }
        }
    }
}

enum MemoryEvent {
    TimestampLatch,
}

#[derive(Clone)]
struct MemoryEventHandler {
    rx: Receiver<MemoryEvent>,
}

impl MemoryEventHandler {
    async fn handle_events(&self, worker: &Worker) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                MemoryEvent::TimestampLatch => self.handle_timestamp_latch(worker).await,
            }
        }
    }

    async fn handle_timestamp_latch(&self, worker: &Worker) {
        let mut memory = worker.memory.lock().await;
        match memory.read::<ABRM::TimestampLatch>() {
            Ok(value) => {
                if value != 1 {
                    return;
                }
            }
            Err(e) => {
                log::warn!("failed to read ABRM::TimestampLatch {}", e);
                return;
            }
        }

        let timestamp_ns = worker.timestamp.as_nanos().await;
        let signal = InterfaceSignal::ToEvent(EventSignal::UpdateTimestamp(timestamp_ns));
        worker.try_send_signal(signal);
        if let Err(e) = memory.write::<ABRM::Timestamp>(timestamp_ns) {
            log::warn!("failed to write to ABRM::Timestamp register {}", e)
        }
    }
}

struct TimestampLatchObserver {
    sender: Sender<MemoryEvent>,
}

impl MemoryObserver for TimestampLatchObserver {
    fn update(&self) {
        if let Err(e) = self.sender.try_send(MemoryEvent::TimestampLatch) {
            log::warn!("memory observer error: {}", e);
        }
    }
}

#[derive(Debug, Error)]
enum ProtocolError {
    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("internal buffer for a packet is something wrong")]
    BufferError(#[from] std::io::Error),
}

type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

/// Command packet parser implementaion.
mod cmd {
    pub(super) use crate::u3v::protocol::cmd::{
        ReadMem, ReadMemStacked, ScdKind, WriteMem, WriteMemStacked,
    };

    use std::io::Cursor;

    use byteorder::{ReadBytesExt, LE};

    use crate::u3v::protocol::{cmd::*, parse_util};

    use super::{ProtocolError, ProtocolResult};

    pub(super) struct CommandPacket<'a> {
        ccd: CommandCcd,
        raw_scd: &'a [u8],
    }

    impl<'a> CommandPacket<'a> {
        const PREFIX_MAGIC: u32 = 0x43563355;

        pub(super) fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf.as_ref());
            Self::parse_prefix(&mut cursor)?;

            let ccd = CommandCcd::parse(&mut cursor)?;
            let raw_scd = &cursor.get_ref()[cursor.position() as usize..];
            Ok(Self { ccd, raw_scd })
        }

        pub(super) fn scd_as<T: ParseScd<'a>>(&self) -> ProtocolResult<T> {
            T::parse(self.raw_scd, &self.ccd)
        }

        pub(super) fn ccd(&self) -> &CommandCcd {
            &self.ccd
        }

        fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<()> {
            let magic = cursor.read_u32::<LE>()?;
            if magic == Self::PREFIX_MAGIC {
                Ok(())
            } else {
                Err(ProtocolError::InvalidPacket("invalid prefix magic".into()))
            }
        }
    }

    impl CommandCcd {
        fn parse<'a>(cursor: &mut Cursor<&'a [u8]>) -> ProtocolResult<Self> {
            let flag = CommandFlag::parse(cursor)?;
            let scd_kind = ScdKind::parse(cursor)?;
            let scd_len = cursor.read_u16::<LE>()?;
            let request_id = cursor.read_u16::<LE>()?;

            Ok(Self::new(flag, scd_kind, scd_len, request_id))
        }
    }

    impl CommandFlag {
        fn parse(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<Self> {
            let raw = cursor.read_u16::<LE>()?;
            if raw == 1 << 14 {
                Ok(Self::RequestAck)
            } else if raw == 1 << 15 {
                Ok(Self::CommandResend)
            } else {
                Err(ProtocolError::InvalidPacket("invalid command flag".into()))
            }
        }
    }

    impl ScdKind {
        fn parse(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<Self> {
            let raw = cursor.read_u16::<LE>()?;
            match raw {
                0x0800 => Ok(Self::ReadMem),
                0x0802 => Ok(Self::WriteMem),
                0x0806 => Ok(Self::ReadMemStacked),
                0x0808 => Ok(Self::WriteMemStacked),
                _ => Err(ProtocolError::InvalidPacket("invalid  command id".into())),
            }
        }
    }

    pub(super) trait ParseScd<'a>: Sized {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self>;
    }

    impl<'a> ParseScd<'a> for ReadMem {
        fn parse(buf: &'a [u8], _ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let address = cursor.read_u64::<LE>()?;
            let reserved = cursor.read_u16::<LE>()?;
            if reserved != 0 {
                return Err(ProtocolError::InvalidPacket(
                    "the reserved field of Read command must be zero".into(),
                ));
            }
            let read_length = cursor.read_u16::<LE>()?;
            Ok(Self::new(address, read_length))
        }
    }

    impl<'a> ParseScd<'a> for WriteMem<'a> {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let address = cursor.read_u64::<LE>()?;
            let data = parse_util::read_bytes(&mut cursor, ccd.scd_len() - 8)?;
            Self::new(address, data)
                .map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))
        }
    }

    impl<'a> ParseScd<'a> for ReadMemStacked {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let mut len = ccd.scd_len();
            let mut entries = Vec::with_capacity(len as usize / 12);
            while len > 0 {
                let address = cursor.read_u64::<LE>()?;
                let reserved = cursor.read_u16::<LE>()?;
                if reserved != 0 {
                    return Err(ProtocolError::InvalidPacket(
                        "the reserved field of ReadMemStacked command must be zero".into(),
                    ));
                }
                let read_length = cursor.read_u16::<LE>()?;
                entries.push(ReadMem::new(address, read_length));

                len -= 12;
            }

            Self::new(entries).map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))
        }
    }

    impl<'a> ParseScd<'a> for WriteMemStacked<'a> {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let mut regs = vec![];
            let mut len = ccd.scd_len();

            while len > 0 {
                let address = cursor.read_u64::<LE>()?;
                let reserved = cursor.read_u16::<LE>()?;
                if reserved != 0 {
                    return Err(ProtocolError::InvalidPacket(
                        "the reserved field of WriteMemStacked command must be zero".into(),
                    ));
                }
                let data_length = cursor.read_u16::<LE>()?;
                let data = parse_util::read_bytes(&mut cursor, data_length)?;
                regs.push(
                    WriteMem::new(address, data)
                        .map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))?,
                );

                len -= 12 + data_length;
            }

            Self::new(regs).map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_read_mem() {
            let cmd = ReadMem::new(0x1f, 16).finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag(), CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind(), ScdKind::ReadMem);
            assert_eq!(parsed_ccd.request_id(), 1);

            let parsed_scd = parsed_cmd.scd_as::<ReadMem>().unwrap();
            assert_eq!(parsed_scd.address, 0x1f);
            assert_eq!(parsed_scd.read_length, 16);
        }

        #[test]
        fn test_write_mem() {
            let data = &[0, 1, 2, 3];
            let cmd = WriteMem::new(0xfff, data).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag(), CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind(), ScdKind::WriteMem);
            assert_eq!(parsed_ccd.request_id(), 1);

            let parsed_scd = parsed_cmd.scd_as::<WriteMem>().unwrap();
            assert_eq!(parsed_scd.address, 0xfff);
            assert_eq!(parsed_scd.data, data);
        }

        #[test]
        fn test_read_mem_stacked() {
            let regs = vec![ReadMem::new(0x0f, 4), ReadMem::new(0xf0, 8)];
            let cmd = ReadMemStacked::new(regs).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag(), CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind(), ScdKind::ReadMemStacked);
            assert_eq!(parsed_ccd.request_id(), 1);

            let parsed_scd = parsed_cmd.scd_as::<ReadMemStacked>().unwrap();
            assert_eq!(parsed_scd.entries[0], ReadMem::new(0x0f, 4));
            assert_eq!(parsed_scd.entries[1], ReadMem::new(0xf0, 8));
        }

        #[test]
        fn test_write_mem_stacked() {
            let data0 = &[0, 1, 2, 3];
            let data1 = &[1, 2, 3, 4, 5, 6, 7];
            let regs = vec![
                WriteMem::new(0x0f, data0).unwrap(),
                WriteMem::new(0xf0, data1).unwrap(),
            ];
            let cmd = WriteMemStacked::new(regs).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag(), CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind(), ScdKind::WriteMemStacked);
            assert_eq!(parsed_ccd.request_id(), 1);

            let parsed_scd = parsed_cmd.scd_as::<WriteMemStacked>().unwrap();
            assert_eq!(parsed_scd.entries[0].address, 0x0f);
            assert_eq!(parsed_scd.entries[0].data, data0);
            assert_eq!(parsed_scd.entries[1].address, 0xf0);
            assert_eq!(parsed_scd.entries[1].data, data1);
        }
    }
}

/// Acknowledge packet serializer implementaion.
mod ack {
    use std::io::Write;
    use std::time;

    use byteorder::{WriteBytesExt, LE};

    use crate::u3v::protocol::{
        ack::{AckCcd, Status, StatusKind},
        cmd,
    };

    use super::ProtocolResult;
    pub(super) use crate::u3v::protocol::ack::{
        GenCpStatus, Pending, ReadMem, ReadMemStacked, ScdKind, UsbSpecificStatus, WriteMem,
        WriteMemStacked,
    };

    pub(super) struct AckPacket<T> {
        pub(super) ccd: AckCcd,
        scd: T,
    }

    impl<T> AckPacket<T>
    where
        T: AckSerialize,
    {
        const PREFIX_MAGIC: u32 = 0x43563355;

        pub(super) fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_u32::<LE>(Self::PREFIX_MAGIC)?;
            self.ccd.serialize(&mut buf)?;
            self.scd.serialize(&mut buf)?;
            Ok(())
        }

        fn from_scd(scd: T, request_id: u16) -> Self {
            let ccd = AckCcd::new(&scd, request_id);
            Self { ccd, scd }
        }
    }

    impl AckCcd {
        fn new(scd: &impl AckSerialize, request_id: u16) -> Self {
            Self {
                status: scd.status(),
                scd_kind: scd.scd_kind(),
                request_id,
                scd_len: scd.scd_len(),
            }
        }

        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_u16::<LE>(self.status().code())?;
            self.scd_kind().serialize(&mut buf)?;
            buf.write_u16::<LE>(self.scd_len())?;
            buf.write_u16::<LE>(self.request_id())?;
            Ok(())
        }
    }

    impl ScdKind {
        fn serialize(self, mut buf: impl Write) -> ProtocolResult<()> {
            let raw = match self {
                Self::ReadMem => 0x0801,
                Self::WriteMem => 0x0803,
                Self::Pending => 0x0805,
                Self::ReadMemStacked => 0x0807,
                Self::WriteMemStacked => 0x0809,
            };

            buf.write_u16::<LE>(raw)?;
            Ok(())
        }
    }

    pub(super) trait AckSerialize: Sized {
        fn serialize(&self, buf: impl Write) -> ProtocolResult<()>;
        fn scd_len(&self) -> u16;
        fn scd_kind(&self) -> ScdKind;

        fn status(&self) -> Status {
            GenCpStatus::Success.into()
        }

        fn finalize(self, request_id: u16) -> AckPacket<Self> {
            AckPacket::from_scd(self, request_id)
        }
    }

    impl<'a> ReadMem<'a> {
        pub(super) fn new(data: &'a [u8]) -> Self {
            debug_assert!(data.len() <= u16::MAX as usize);
            Self { data }
        }
    }

    impl<'a> AckSerialize for ReadMem<'a> {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_all(&self.data)?;
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            self.data.len() as u16
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::ReadMem
        }
    }

    impl WriteMem {
        pub(super) fn new(length: u16) -> Self {
            Self { length }
        }
    }

    impl<'a> AckSerialize for WriteMem {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_u16::<LE>(0)?;
            buf.write_u16::<LE>(self.length)?;
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            4
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::WriteMem
        }
    }

    impl Pending {
        pub(super) fn _new(timeout: time::Duration) -> Self {
            debug_assert!(timeout.as_millis() <= std::u16::MAX as u128);
            Self { timeout }
        }
    }

    impl AckSerialize for Pending {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_u16::<LE>(0)?;
            buf.write_u16::<LE>(self.timeout.as_millis() as u16)?;
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            4
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::Pending
        }
    }

    impl<'a> ReadMemStacked<'a> {
        pub(super) fn _new(data: &'a [u8]) -> Self {
            debug_assert!(data.len() <= u16::MAX as usize);
            Self { data }
        }
    }

    impl<'a> AckSerialize for ReadMemStacked<'a> {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_all(&self.data)?;
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            self.data.len() as u16
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::ReadMemStacked
        }
    }

    impl WriteMemStacked {
        pub(super) fn _new(lengths: Vec<u16>) -> Self {
            debug_assert!(Self::scd_len(&lengths) <= u16::MAX as usize);
            Self { lengths }
        }

        fn scd_len(lengths: &[u16]) -> usize {
            lengths.len() * 4
        }
    }

    impl AckSerialize for WriteMemStacked {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            for len in &self.lengths {
                buf.write_u16::<LE>(0)?;
                buf.write_u16::<LE>(*len)?;
            }

            Ok(())
        }

        fn scd_len(&self) -> u16 {
            Self::scd_len(&self.lengths) as u16
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::WriteMemStacked
        }
    }

    pub(super) struct ErrorAck {
        status: Status,
        scd_kind: ScdKind,
    }

    impl ErrorAck {
        pub(super) fn new(status: impl Into<Status>, scd_kind: impl Into<ScdKind>) -> Self {
            Self {
                status: status.into(),
                scd_kind: scd_kind.into(),
            }
        }
    }

    impl AckSerialize for ErrorAck {
        fn serialize(&self, _buf: impl Write) -> ProtocolResult<()> {
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            0
        }

        fn scd_kind(&self) -> ScdKind {
            self.scd_kind
        }

        fn status(&self) -> Status {
            self.status
        }
    }

    impl GenCpStatus {
        fn as_code(self) -> u16 {
            use GenCpStatus::*;
            match self {
                Success => 0x0000,
                NotImplemented => 0x8001,
                InvalidParameter => 0x8002,
                InvalidAddress => 0x8003,
                WriteProtect => 0x8004,
                BadAlignment => 0x8005,
                AccessDenied => 0x8006,
                Busy => 0x8007,
                Timeout => 0x800B,
                InvalidHeader => 0x800E,
                WrongConfig => 0x800F,
                GenericError => 0x8FFF,
            }
        }
    }

    impl UsbSpecificStatus {
        fn as_code(self) -> u16 {
            use UsbSpecificStatus::*;
            match self {
                ResendNotSupported => 0xA001,
                StreamEndpointHalted => 0xA002,
                PayloadSizeNotAligned => 0xA003,
                InvalidSiState => 0xA004,
                EventEndpointHalted => 0xA005,
            }
        }
    }

    impl From<GenCpStatus> for Status {
        fn from(cp_status: GenCpStatus) -> Status {
            let code = cp_status.as_code();
            let kind = StatusKind::GenCp(cp_status);
            Self { code, kind }
        }
    }

    impl From<UsbSpecificStatus> for Status {
        fn from(usb_status: UsbSpecificStatus) -> Status {
            let code = usb_status.as_code();
            let kind = StatusKind::UsbSpecific(usb_status);
            Self { code, kind }
        }
    }

    impl From<cmd::ScdKind> for ScdKind {
        fn from(kind: cmd::ScdKind) -> Self {
            match kind {
                cmd::ScdKind::ReadMem => ScdKind::ReadMem,
                cmd::ScdKind::WriteMem => ScdKind::WriteMem,
                cmd::ScdKind::ReadMemStacked => ScdKind::ReadMemStacked,
                cmd::ScdKind::WriteMemStacked => ScdKind::WriteMemStacked,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::u3v::protocol::ack as host_side_ack;

        #[test]
        fn test_read_mem() {
            let data = &[1, 2, 3, 4];
            let command = ReadMem::new(data).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::ReadMem);
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<ReadMem>().unwrap();
            assert_eq!(parsed_scd.data, data);
        }

        #[test]
        fn test_write_mem() {
            let command = WriteMem::new(16).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::WriteMem);
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<WriteMem>().unwrap();
            assert_eq!(parsed_scd.length, 16);
        }

        #[test]
        fn test_pending() {
            let timeout = time::Duration::from_millis(700);
            let command = Pending::_new(timeout).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::Pending);
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<Pending>().unwrap();
            assert_eq!(parsed_scd.timeout, timeout);
        }

        #[test]
        fn test_read_mem_stacked() {
            let data = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
            let command = ReadMemStacked::_new(data).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::ReadMemStacked);
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<ReadMemStacked>().unwrap();
            assert_eq!(parsed_scd.data, data);
        }

        #[test]
        fn test_write_mem_stacked() {
            let lengths = vec![8, 16];
            let command = WriteMemStacked::_new(lengths.clone()).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::WriteMemStacked);
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<WriteMemStacked>().unwrap();
            assert_eq!(parsed_scd.lengths, lengths);
        }

        #[test]
        fn test_gencp_error() {
            let err_status = GenCpStatus::AccessDenied;
            let command = ErrorAck::new(err_status, ScdKind::ReadMem).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            let status = parsed.status();
            assert!(!status.is_success());
            assert_eq!(status.kind, StatusKind::GenCp(err_status));
        }

        #[test]
        fn test_u3v_error() {
            let err_status = UsbSpecificStatus::StreamEndpointHalted;
            let command = ErrorAck::new(err_status, ScdKind::ReadMem).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            let status = parsed.status();
            assert!(!status.is_success());
            assert_eq!(status.kind, StatusKind::UsbSpecific(err_status));
        }
    }
}
