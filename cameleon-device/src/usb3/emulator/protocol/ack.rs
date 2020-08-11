// TODO: Implement status kind builder.
use std::io::Write;
use std::time;

use byteorder::{WriteBytesExt, LE};

use crate::usb3::protocol::ack::*;

use super::super::EmulatorResult;

pub(super) struct AckPacket<T> {
    ccd: AckCcd,
    scd: T,
}

impl<T> AckPacket<T>
where
    T: AckSerialize,
{
    const PREFIX_MAGIC: u32 = 0x43563355;

    pub(super) fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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

    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
        buf.write_u16::<LE>(self.status.code())?;
        self.scd_kind.serialize(&mut buf)?;
        buf.write_u16::<LE>(self.scd_len)?;
        buf.write_u16::<LE>(self.request_id)?;
        Ok(())
    }
}

impl ScdKind {
    fn serialize(self, mut buf: impl Write) -> EmulatorResult<()> {
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
    fn serialize(&self, buf: impl Write) -> EmulatorResult<()>;
    fn scd_len(&self) -> u16;
    fn scd_kind(&self) -> ScdKind;
    fn status(&self) -> Status {
        // TODO: Implement status kind builder.
        Status {
            code: 0x0000,
            kind: StatusKind::GenCp(GenCpStatus::Success),
        }
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
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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
    pub(super) fn new(timeout: time::Duration) -> Self {
        debug_assert!(timeout.as_millis() <= std::u16::MAX as u128);
        Self { timeout }
    }
}

impl AckSerialize for Pending {
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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
    pub(super) fn new(data: &'a [u8]) -> Self {
        debug_assert!(data.len() <= u16::MAX as usize);
        Self { data }
    }
}

impl<'a> AckSerialize for ReadMemStacked<'a> {
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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
    pub(super) fn new(lengths: Vec<u16>) -> Self {
        debug_assert!(Self::scd_len(&lengths) <= u16::MAX as usize);
        Self { lengths }
    }

    fn scd_len(lengths: &[u16]) -> usize {
        lengths.len() * 4
    }
}

impl AckSerialize for WriteMemStacked {
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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

pub(super) struct CustomAck<'a> {
    command_id: u16,
    data: &'a [u8],
}

impl<'a> CustomAck<'a> {
    pub(super) fn new(command_id: u16, data: &'a [u8]) -> Self {
        debug_assert!(data.len() <= u16::MAX as usize);
        debug_assert!(ScdKind::is_custom(command_id));
        Self { command_id, data }
    }
}

impl<'a> AckSerialize for CustomAck<'a> {
    fn serialize(&self, mut buf: impl Write) -> EmulatorResult<()> {
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

pub(super) struct ErrorAck {
    status: Status,
    scd_kind: ScdKind,
}
impl ErrorAck {
    pub(super) fn new(status: Status, scd_kind: ScdKind) -> Self {
        Self { status, scd_kind }
    }
}

impl AckSerialize for ErrorAck {
    fn serialize(&self, _buf: impl Write) -> EmulatorResult<()> {
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

    // TODO: Add tests for ErrorAck.
}
