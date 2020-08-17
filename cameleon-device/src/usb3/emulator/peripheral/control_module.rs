use std::borrow::Cow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use async_std::{
    prelude::*,
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use futures::{channel::oneshot, select, FutureExt};
use thiserror::Error;

use super::{
    fake_protocol::IfaceKind,
    interface::IfaceState,
    memory::{Memory, MemoryError},
    signal::*,
};

use ack::AckSerialize;

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
    pub(super) async fn run(self) {
        let mut req_rx = self.req_rx.fuse();
        let mut inner_event_rx = self.ctrl_manage_rx.fuse();
        let mut worker_manager = WorkerManager::new(
            self.ack_tx.clone(),
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

        // Shutdown event module and stream module.
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
    ack_tx: Sender<Vec<u8>>,
    memory: Arc<Mutex<Memory>>,
    completed: Sender<()>,
    on_processing: Arc<AtomicBool>,
    iface_state: IfaceState,
    event_manage_tx: Sender<EventManagementSignal>,
    stream_manage_tx: Sender<StreamManagementSignal>,
}

impl Worker {
    // TODO: Emulate pending situation.
    async fn run(self, command: Vec<u8>) {
        let cmd_packet = match self.try_parse_command(&command) {
            Some(packet) => packet,
            None => return,
        };
        let ccd = cmd_packet.ccd();

        // If another interface is halted, control module must notify it to the host.
        if self.iface_state.is_halt(IfaceKind::Event).await {
            let ack = ack::ErrorAck::new(ack::UsbSpecificStatus::EventEndpointHalted, ccd.scd_kind)
                .finalize(ccd.request_id);
            self.try_send_ack(ack);
            return;
        } else if self.iface_state.is_halt(IfaceKind::Stream).await {
            let ack =
                ack::ErrorAck::new(ack::UsbSpecificStatus::StreamEndpointHalted, ccd.scd_kind)
                    .finalize(ccd.request_id);
            self.try_send_ack(ack);
            return;
        }

        // If another thread is processing command simultaneously, return busy error ack.
        if self
            .on_processing
            .compare_and_swap(false, true, Ordering::Relaxed)
        {
            let ack =
                ack::ErrorAck::new(ack::GenCpStatus::Busy, ccd.scd_kind).finalize(ccd.request_id);
            self.try_send_ack(ack);
        }

        match ccd.scd_kind {
            cmd::ScdKind::ReadMem => self.process_read_mem(cmd_packet),
            cmd::ScdKind::WriteMem => self.process_write_mem(cmd_packet),
            cmd::ScdKind::ReadMemStacked => self.process_read_mem_stacked(cmd_packet),
            cmd::ScdKind::WriteMemStacked => self.process_write_mem_stacked(cmd_packet),
            cmd::ScdKind::Custom(_) => self.process_custom(cmd_packet),
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
                let mut buf = vec![];

                if let Err(e) = ack.serialize(&mut buf) {
                    log::error!("{}", e);
                    return None;
                }

                match self.ack_tx.try_send(buf) {
                    Ok(()) => {}
                    Err(e) => {
                        log::warn!("can't push internal acknowledge packet queue. cause: {}", e)
                    }
                }

                None
            }
        }
    }

    fn process_read_mem(&self, command: cmd::CommandPacket) {
        let scd: cmd::ReadMem = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id;
        let scd_kind = ccd.scd_kind;

        let memory = task::block_on(self.memory.lock());
        let address = scd.address as usize;
        let read_length = scd.read_length as usize;
        match memory.read_mem(address..address + read_length) {
            Ok(data) => {
                let ack = ack::ReadMem::new(data).finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::InvalidAddress) => {
                let ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidAddress, scd_kind)
                    .finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::AddressNotReadable) => {
                let ack = ack::ErrorAck::new(ack::GenCpStatus::AccessDenied, scd_kind)
                    .finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::AddressNotWritable) => unreachable!(),
        };
    }

    fn process_write_mem(&self, command: cmd::CommandPacket) {
        let scd: cmd::WriteMem = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id;
        let scd_kind = ccd.scd_kind;

        let mut memory = task::block_on(self.memory.lock());
        match memory.write_mem(scd.address as usize, scd.data) {
            Ok(()) => {
                let ack = ack::WriteMem::new(scd.data.len() as u16).finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::InvalidAddress) => {
                let ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidAddress, scd_kind)
                    .finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::AddressNotWritable) => {
                let ack = ack::ErrorAck::new(ack::GenCpStatus::WriteProtect, scd_kind)
                    .finalize(req_id);
                self.try_send_ack(ack);
            }
            Err(MemoryError::AddressNotReadable) => unreachable!(),
        }
    }

    fn process_read_mem_stacked(&self, command: cmd::CommandPacket) {
        let scd: cmd::WriteMemStacked = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id;
        let scd_kind = ccd.scd_kind;

        // TODO: Should we implemnent this command?
        let ack = ack::ErrorAck::new(ack::GenCpStatus::NotImplemented, scd_kind).finalize(req_id);
        self.try_send_ack(ack);
    }

    fn process_write_mem_stacked(&self, command: cmd::CommandPacket) {
        let scd: cmd::WriteMemStacked = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id;
        let scd_kind = ccd.scd_kind;

        // TODO: Should we implemnent this command?
        let ack = ack::ErrorAck::new(ack::GenCpStatus::NotImplemented, scd_kind).finalize(req_id);
        self.try_send_ack(ack);
    }

    fn process_custom(&self, command: cmd::CommandPacket) {
        let scd: cmd::WriteMemStacked = match self.try_extract_scd(&command) {
            Some(scd) => scd,
            None => return,
        };
        let ccd = command.ccd();
        let req_id = ccd.request_id;
        let scd_kind = ccd.scd_kind;

        // TODO: Should we implemnent this command?
        let ack = ack::ErrorAck::new(ack::GenCpStatus::NotImplemented, scd_kind).finalize(req_id);
        self.try_send_ack(ack);
    }

    fn try_extract_scd<'a, T>(&self, command: &cmd::CommandPacket<'a>) -> Option<T>
    where
        T: cmd::ParseScd<'a>,
    {
        match command.scd_as::<T>() {
            Ok(scd) => Some(scd),
            Err(e) => {
                let ccd = command.ccd();
                let ack = ack::ErrorAck::new(ack::GenCpStatus::InvalidParameter, ccd.scd_kind)
                    .finalize(ccd.request_id);
                self.try_send_ack(ack);
                None
            }
        }
    }

    fn try_send_ack<T>(&self, ack: ack::AckPacket<T>)
    where
        T: AckSerialize,
    {
        let mut buf = vec![];

        if let Err(e) = ack.serialize(&mut buf) {
            log::error!("{}", e);
        }

        match self.ack_tx.try_send(buf) {
            Ok(()) => {}
            Err(e) => log::warn!("can't push internal acknowledge packet queue. cause: {}", e),
        }
    }
}

struct WorkerManager {
    tx: Sender<()>,
    rx: Receiver<()>,

    ack_tx: Sender<Vec<u8>>,
    memory: Arc<Mutex<Memory>>,
    iface_state: IfaceState,
    on_processing: Arc<AtomicBool>,
    event_manage_tx: Sender<EventManagementSignal>,
    stream_manage_tx: Sender<StreamManagementSignal>,
}

impl WorkerManager {
    fn new(
        ack_tx: Sender<Vec<u8>>,
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
            ack_tx,
            memory,
            iface_state,
            on_processing,
            event_manage_tx,
            stream_manage_tx,
        }
    }

    fn worker(&self) -> Worker {
        Worker {
            ack_tx: self.ack_tx.clone(),
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
    use std::io::Cursor;

    use byteorder::{ReadBytesExt, LE};

    use crate::usb3::protocol::{command::*, parse_util};

    use super::{ProtocolError, ProtocolResult};
    pub(super) use crate::usb3::protocol::command::{
        CustomCommand, ReadMem, ReadMemStacked, ScdKind, WriteMem, WriteMemStacked,
    };

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

            Ok(Self {
                flag,
                scd_kind,
                scd_len: scd_len,
                request_id,
            })
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
                custom if ScdKind::is_custom(raw) => Ok(Self::Custom(custom)),
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
            let data = parse_util::read_bytes(&mut cursor, ccd.scd_len - 8)?;
            Self::new(address, data)
                .map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))
        }
    }

    impl<'a> ParseScd<'a> for ReadMemStacked {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let mut len = ccd.scd_len;
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
            let mut entries = vec![];
            let mut len = ccd.scd_len;

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
                entries.push(
                    WriteMem::new(address, data)
                        .map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))?,
                );

                len -= 12 + data_length;
            }

            Self::new(entries).map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))
        }
    }

    impl<'a> ParseScd<'a> for CustomCommand<'a> {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let command_id = match ccd.scd_kind {
                ScdKind::Custom(id) => id,
                _ => panic!("not custome ccd"),
            };

            let mut cursor = Cursor::new(buf);
            let data = parse_util::read_bytes(&mut cursor, ccd.scd_len)?;
            Ok(CustomCommand::new(command_id, data)
                .map_err(|err| ProtocolError::InvalidPacket(err.to_string().into()))?)
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
            assert_eq!(parsed_ccd.flag, CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind, ScdKind::ReadMem);
            assert_eq!(parsed_ccd.request_id, 1);

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
            assert_eq!(parsed_ccd.flag, CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind, ScdKind::WriteMem);
            assert_eq!(parsed_ccd.request_id, 1);

            let parsed_scd = parsed_cmd.scd_as::<WriteMem>().unwrap();
            assert_eq!(parsed_scd.address, 0xfff);
            assert_eq!(parsed_scd.data, data);
        }

        #[test]
        fn test_read_mem_stacked() {
            let entries = vec![ReadMem::new(0x0f, 4), ReadMem::new(0xf0, 8)];
            let cmd = ReadMemStacked::new(entries).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag, CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind, ScdKind::ReadMemStacked);
            assert_eq!(parsed_ccd.request_id, 1);

            let parsed_scd = parsed_cmd.scd_as::<ReadMemStacked>().unwrap();
            assert_eq!(parsed_scd.entries[0], ReadMem::new(0x0f, 4));
            assert_eq!(parsed_scd.entries[1], ReadMem::new(0xf0, 8));
        }

        #[test]
        fn test_write_mem_stacked() {
            let data0 = &[0, 1, 2, 3];
            let data1 = &[1, 2, 3, 4, 5, 6, 7];
            let entries = vec![
                WriteMem::new(0x0f, data0).unwrap(),
                WriteMem::new(0xf0, data1).unwrap(),
            ];
            let cmd = WriteMemStacked::new(entries).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag, CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind, ScdKind::WriteMemStacked);
            assert_eq!(parsed_ccd.request_id, 1);

            let parsed_scd = parsed_cmd.scd_as::<WriteMemStacked>().unwrap();
            assert_eq!(parsed_scd.entries[0].address, 0x0f);
            assert_eq!(parsed_scd.entries[0].data, data0);
            assert_eq!(parsed_scd.entries[1].address, 0xf0);
            assert_eq!(parsed_scd.entries[1].data, data1);
        }

        #[test]
        fn test_custom_cmd() {
            let data = &[0, 1, 2];
            let cmd = CustomCommand::new(0xfff0, data).unwrap().finalize(1);
            let mut buf = vec![];
            cmd.serialize(&mut buf).unwrap();

            let parsed_cmd = CommandPacket::parse(&buf).unwrap();
            let parsed_ccd = &parsed_cmd.ccd;
            assert_eq!(parsed_ccd.flag, CommandFlag::RequestAck);
            assert_eq!(parsed_ccd.scd_kind, ScdKind::Custom(0xfff0));
            assert_eq!(parsed_ccd.request_id, 1);

            let parsed_scd = parsed_cmd.scd_as::<CustomCommand>().unwrap();
            assert_eq!(parsed_scd.data, data);
        }
    }
}

/// Acknowledge packet serializer implementaion.
mod ack {
    use std::io::Write;
    use std::time;

    use byteorder::{WriteBytesExt, LE};

    use crate::usb3::protocol::{
        ack::{AckCcd, Status, StatusKind},
        command,
    };

    use super::ProtocolResult;
    pub(super) use crate::usb3::protocol::ack::{
        GenCpStatus, ReadMem, ReadMemStacked, ScdKind, UsbSpecificStatus, WriteMem, Pending,
        WriteMemStacked,
    };

    pub(super) struct AckPacket<T> {
        ccd: AckCcd,
        scd: T,
    }

    impl<T> AckPacket<T>
    where
        T: AckSerialize,
    {
        const PREFIX_MAGIC: u32 = 0x43563355;

        pub(crate) fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
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
            buf.write_u16::<LE>(self.status.code())?;
            self.scd_kind.serialize(&mut buf)?;
            buf.write_u16::<LE>(self.scd_len)?;
            buf.write_u16::<LE>(self.request_id)?;
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
                Self::Custom(raw) => {
                    debug_assert!(ScdKind::is_custom(raw));
                    raw
                }
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
        pub(crate) fn new(data: &'a [u8]) -> Self {
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
        pub(crate) fn new(length: u16) -> Self {
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
        pub(crate) fn new(timeout: time::Duration) -> Self {
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
        pub(crate) fn new(data: &'a [u8]) -> Self {
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
        pub(crate) fn new(lengths: Vec<u16>) -> Self {
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

    pub(crate) struct CustomAck<'a> {
        command_id: u16,
        data: &'a [u8],
    }

    impl<'a> CustomAck<'a> {
        pub(crate) fn new(command_id: u16, data: &'a [u8]) -> Self {
            debug_assert!(data.len() <= u16::MAX as usize);
            debug_assert!(ScdKind::is_custom(command_id));
            Self { command_id, data }
        }
    }

    impl<'a> AckSerialize for CustomAck<'a> {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_all(self.data)?;
            Ok(())
        }

        fn scd_len(&self) -> u16 {
            self.data.len() as u16
        }

        fn scd_kind(&self) -> ScdKind {
            ScdKind::Custom(self.command_id)
        }
    }

    pub(crate) struct ErrorAck {
        status: Status,
        scd_kind: ScdKind,
    }

    impl ErrorAck {
        pub(crate) fn new(status: impl Into<Status>, scd_kind: impl Into<ScdKind>) -> Self {
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

    impl From<command::ScdKind> for ScdKind {
        fn from(kind: command::ScdKind) -> Self {
            match kind {
                command::ScdKind::ReadMem => ScdKind::ReadMem,
                command::ScdKind::WriteMem => ScdKind::WriteMem,
                command::ScdKind::ReadMemStacked => ScdKind::ReadMemStacked,
                command::ScdKind::WriteMemStacked => ScdKind::WriteMemStacked,
                command::ScdKind::Custom(code) => ScdKind::Custom(code | 1),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::usb3::protocol::ack as host_side_ack;

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
            let command = Pending::new(timeout).finalize(1);
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
            let command = ReadMemStacked::new(data).finalize(1);
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
            let command = WriteMemStacked::new(lengths.clone()).finalize(1);
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
        fn test_custom() {
            let code = 0xff01;
            let data = &[0, 1, 2, 3];
            let command = CustomAck::new(code, data).finalize(1);
            let mut buf = vec![];
            command.serialize(&mut buf).unwrap();

            let parsed = host_side_ack::AckPacket::parse(&buf).unwrap();
            assert!(parsed.status().is_success());
            assert_eq!(parsed.scd_kind(), ScdKind::Custom(0xff01));
            assert_eq!(parsed.request_id(), 1);

            let parsed_scd = parsed.scd_as::<host_side_ack::CustomAck>().unwrap();
            assert_eq!(parsed_scd.data, data);
        }

        #[test]
        fn test_gen_cp_error() {
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
        fn test_usb3_error() {
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
