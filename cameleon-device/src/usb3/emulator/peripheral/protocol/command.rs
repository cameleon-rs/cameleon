use std::io::Cursor;

use byteorder::{ReadBytesExt, LE};

use crate::usb3::protocol::{command::*, parse_util};

use super::super::{EmulatorError, EmulatorResult};

pub(super) struct CommandPacket<'a> {
    ccd: CommandCcd,
    raw_scd: &'a [u8],
}

impl<'a> CommandPacket<'a> {
    const PREFIX_MAGIC: u32 = 0x43563355;

    pub(super) fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> EmulatorResult<Self> {
        let mut cursor = Cursor::new(buf.as_ref());
        Self::parse_prefix(&mut cursor)?;

        let ccd = CommandCcd::parse(&mut cursor)?;
        let raw_scd = &cursor.get_ref()[cursor.position() as usize..];
        Ok(Self { ccd, raw_scd })
    }

    pub(super) fn scd_as<T: ParseScd<'a>>(&self) -> EmulatorResult<T> {
        T::parse(self.raw_scd, &self.ccd)
    }

    pub(super) fn ccd(&self) -> &CommandCcd {
        &self.ccd
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> EmulatorResult<()> {
        let magic = cursor.read_u32::<LE>()?;
        if magic == Self::PREFIX_MAGIC {
            Ok(())
        } else {
            Err(EmulatorError::InvalidPacket("invalid prefix magic".into()))
        }
    }
}

impl CommandCcd {
    fn parse<'a>(cursor: &mut Cursor<&'a [u8]>) -> EmulatorResult<Self> {
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
    fn parse(cursor: &mut Cursor<&[u8]>) -> EmulatorResult<Self> {
        let raw = cursor.read_u16::<LE>()?;
        if raw == 1 << 14 {
            Ok(Self::RequestAck)
        } else if raw == 1 << 15 {
            Ok(Self::CommandResend)
        } else {
            Err(EmulatorError::InvalidPacket("invalid command flag".into()))
        }
    }
}

impl ScdKind {
    fn parse(cursor: &mut Cursor<&[u8]>) -> EmulatorResult<Self> {
        let raw = cursor.read_u16::<LE>()?;
        match raw {
            0x0800 => Ok(Self::ReadMem),
            0x0802 => Ok(Self::WriteMem),
            0x0806 => Ok(Self::ReadMemStacked),
            0x0808 => Ok(Self::WriteMemStacked),
            custom if ScdKind::is_custom(raw) => Ok(Self::Custom(custom)),
            _ => Err(EmulatorError::InvalidPacket("invalid  command id".into())),
        }
    }
}

pub(super) trait ParseScd<'a>: Sized {
    fn parse(buf: &'a [u8], ccd: &CommandCcd) -> EmulatorResult<Self>;
}

impl<'a> ParseScd<'a> for ReadMem {
    fn parse(buf: &'a [u8], _ccd: &CommandCcd) -> EmulatorResult<Self> {
        let mut cursor = Cursor::new(buf);
        let address = cursor.read_u64::<LE>()?;
        let reserved = cursor.read_u16::<LE>()?;
        if reserved != 0 {
            return Err(EmulatorError::InvalidPacket(
                "the reserved field of Read command must be zero".into(),
            ));
        }
        let read_length = cursor.read_u16::<LE>()?;
        Ok(Self::new(address, read_length))
    }
}

impl<'a> ParseScd<'a> for WriteMem<'a> {
    fn parse(buf: &'a [u8], ccd: &CommandCcd) -> EmulatorResult<Self> {
        let mut cursor = Cursor::new(buf);
        let address = cursor.read_u64::<LE>()?;
        let data = parse_util::read_bytes(&mut cursor, ccd.scd_len - 8)?;
        Self::new(address, data).map_err(|err| EmulatorError::InvalidPacket(err.to_string().into()))
    }
}

impl<'a> ParseScd<'a> for ReadMemStacked {
    fn parse(buf: &'a [u8], ccd: &CommandCcd) -> EmulatorResult<Self> {
        let mut cursor = Cursor::new(buf);
        let mut len = ccd.scd_len;
        let mut entries = Vec::with_capacity(len as usize / 12);
        while len > 0 {
            let address = cursor.read_u64::<LE>()?;
            let reserved = cursor.read_u16::<LE>()?;
            if reserved != 0 {
                return Err(EmulatorError::InvalidPacket(
                    "the reserved field of ReadMemStacked command must be zero".into(),
                ));
            }
            let read_length = cursor.read_u16::<LE>()?;
            entries.push(ReadMem::new(address, read_length));

            len -= 12;
        }

        Self::new(entries).map_err(|err| EmulatorError::InvalidPacket(err.to_string().into()))
    }
}

impl<'a> ParseScd<'a> for WriteMemStacked<'a> {
    fn parse(buf: &'a [u8], ccd: &CommandCcd) -> EmulatorResult<Self> {
        let mut cursor = Cursor::new(buf);
        let mut entries = vec![];
        let mut len = ccd.scd_len;

        while len > 0 {
            let address = cursor.read_u64::<LE>()?;
            let reserved = cursor.read_u16::<LE>()?;
            if reserved != 0 {
                return Err(EmulatorError::InvalidPacket(
                    "the reserved field of WriteMemStacked command must be zero".into(),
                ));
            }
            let data_length = cursor.read_u16::<LE>()?;
            let data = parse_util::read_bytes(&mut cursor, data_length)?;
            entries.push(
                WriteMem::new(address, data)
                    .map_err(|err| EmulatorError::InvalidPacket(err.to_string().into()))?,
            );

            len -= 12 + data_length;
        }

        Self::new(entries).map_err(|err| EmulatorError::InvalidPacket(err.to_string().into()))
    }
}

impl<'a> ParseScd<'a> for CustomCommand<'a> {
    fn parse(buf: &'a [u8], ccd: &CommandCcd) -> EmulatorResult<Self> {
        let command_id = match ccd.scd_kind {
            ScdKind::Custom(id) => id,
            _ => panic!("not custome ccd"),
        };

        let mut cursor = Cursor::new(buf);
        let data = parse_util::read_bytes(&mut cursor, ccd.scd_len)?;
        Ok(CustomCommand::new(command_id, data)
            .map_err(|err| EmulatorError::InvalidPacket(err.to_string().into()))?)
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
