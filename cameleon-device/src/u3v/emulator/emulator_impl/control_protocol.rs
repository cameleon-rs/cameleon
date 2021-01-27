use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum ProtocolError {
    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("internal buffer for a packet is something wrong")]
    BufferError(#[from] std::io::Error),
}

pub(super) type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

/// Command packet parser implementaion.
pub(super) mod cmd {
    pub(in super::super) use crate::u3v::protocol::cmd::{
        ReadMem, ReadMemStacked, ScdKind, WriteMem, WriteMemStacked,
    };

    use std::io::Cursor;

    use crate::u3v::protocol::{cmd::*, parse_util};

    use super::{ProtocolError, ProtocolResult};

    use crate::u3v::protocol::parse_util::ReadBytes;

    pub(in super::super) struct CommandPacket<'a> {
        ccd: CommandCcd,
        raw_scd: &'a [u8],
    }

    impl<'a> CommandPacket<'a> {
        const PREFIX_MAGIC: u32 = 0x43563355;

        pub(in super::super) fn parse(
            buf: &'a (impl AsRef<[u8]> + ?Sized),
        ) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf.as_ref());
            Self::parse_prefix(&mut cursor)?;

            let ccd = CommandCcd::parse(&mut cursor)?;
            let raw_scd = &cursor.get_ref()[cursor.position() as usize..];
            Ok(Self { ccd, raw_scd })
        }

        pub(in super::super) fn scd_as<T: ParseScd<'a>>(&self) -> ProtocolResult<T> {
            T::parse(self.raw_scd, &self.ccd)
        }

        pub(in super::super) fn ccd(&self) -> &CommandCcd {
            &self.ccd
        }

        fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<()> {
            let magic: u32 = cursor.read_bytes()?;
            if magic == Self::PREFIX_MAGIC {
                Ok(())
            } else {
                Err(ProtocolError::InvalidPacket("invalid prefix magic".into()))
            }
        }
    }

    impl CommandCcd {
        fn parse(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<Self> {
            let flag = CommandFlag::parse(cursor)?;
            let scd_kind = ScdKind::parse(cursor)?;
            let scd_len = cursor.read_bytes()?;
            let request_id = cursor.read_bytes()?;

            Ok(Self::new(flag, scd_kind, scd_len, request_id))
        }
    }

    impl CommandFlag {
        fn parse(cursor: &mut Cursor<&[u8]>) -> ProtocolResult<Self> {
            let raw: u16 = cursor.read_bytes()?;
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
            let raw: u16 = cursor.read_bytes()?;
            match raw {
                0x0800 => Ok(Self::ReadMem),
                0x0802 => Ok(Self::WriteMem),
                0x0806 => Ok(Self::ReadMemStacked),
                0x0808 => Ok(Self::WriteMemStacked),
                _ => Err(ProtocolError::InvalidPacket("invalid  command id".into())),
            }
        }
    }

    pub(in super::super) trait ParseScd<'a>: Sized {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self>;
    }

    impl<'a> ParseScd<'a> for ReadMem {
        fn parse(buf: &'a [u8], _ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let address = cursor.read_bytes()?;
            let reserved: u16 = cursor.read_bytes()?;
            if reserved != 0 {
                return Err(ProtocolError::InvalidPacket(
                    "the reserved field of Read command must be zero".into(),
                ));
            }
            let read_length = cursor.read_bytes()?;
            Ok(Self::new(address, read_length))
        }
    }

    impl<'a> ParseScd<'a> for WriteMem<'a> {
        fn parse(buf: &'a [u8], ccd: &CommandCcd) -> ProtocolResult<Self> {
            let mut cursor = Cursor::new(buf);
            let address = cursor.read_bytes()?;
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
                let address = cursor.read_bytes()?;
                let reserved: u16 = cursor.read_bytes()?;
                if reserved != 0 {
                    return Err(ProtocolError::InvalidPacket(
                        "the reserved field of ReadMemStacked command must be zero".into(),
                    ));
                }
                let read_length = cursor.read_bytes()?;
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
                let address = cursor.read_bytes()?;
                let reserved: u16 = cursor.read_bytes()?;
                if reserved != 0 {
                    return Err(ProtocolError::InvalidPacket(
                        "the reserved field of WriteMemStacked command must be zero".into(),
                    ));
                }
                let data_length = cursor.read_bytes()?;
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
pub(super) mod ack {
    use std::io::Write;
    use std::time;

    use crate::u3v::protocol::{
        ack::{AckCcd, Status, StatusKind},
        cmd,
        parse_util::WriteBytes,
    };

    use super::ProtocolResult;
    pub(in super::super) use crate::u3v::protocol::ack::{
        GenCpStatus, Pending, ReadMem, ReadMemStacked, ScdKind, UsbSpecificStatus, WriteMem,
        WriteMemStacked,
    };

    pub(in super::super) struct AckPacket<T> {
        pub(in super::super) ccd: AckCcd,
        scd: T,
    }

    impl<T> AckPacket<T>
    where
        T: AckSerialize,
    {
        const PREFIX_MAGIC: u32 = 0x43563355;

        pub(in super::super) fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_bytes(Self::PREFIX_MAGIC)?;
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
            buf.write_bytes(self.status().code())?;
            self.scd_kind().serialize(&mut buf)?;
            buf.write_bytes(self.scd_len())?;
            buf.write_bytes(self.request_id())?;
            Ok(())
        }
    }

    impl ScdKind {
        fn serialize(self, mut buf: impl Write) -> ProtocolResult<()> {
            let raw: u16 = match self {
                Self::ReadMem => 0x0801,
                Self::WriteMem => 0x0803,
                Self::Pending => 0x0805,
                Self::ReadMemStacked => 0x0807,
                Self::WriteMemStacked => 0x0809,
            };

            buf.write_bytes(raw)?;
            Ok(())
        }
    }

    pub(in super::super) trait AckSerialize: Sized {
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
        pub(in super::super) fn new(data: &'a [u8]) -> Self {
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
        pub(in super::super) fn new(length: u16) -> Self {
            Self { length }
        }
    }

    impl<'a> AckSerialize for WriteMem {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_bytes(0u16)?;
            buf.write_bytes(self.length)?;
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
        pub(in super::super) fn _new(timeout: time::Duration) -> Self {
            debug_assert!(timeout.as_millis() <= std::u16::MAX as u128);
            Self { timeout }
        }
    }

    impl AckSerialize for Pending {
        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_bytes(0u16)?;
            buf.write_bytes(self.timeout.as_millis() as u16)?;
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
        pub(in super::super) fn _new(data: &'a [u8]) -> Self {
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
        pub(in super::super) fn _new(lengths: Vec<u16>) -> Self {
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
                buf.write_bytes(0u16)?;
                buf.write_bytes(*len)?;
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

    pub(in super::super) struct ErrorAck {
        status: Status,
        scd_kind: ScdKind,
    }

    impl ErrorAck {
        pub(in super::super) fn new(
            status: impl Into<Status>,
            scd_kind: impl Into<ScdKind>,
        ) -> Self {
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
