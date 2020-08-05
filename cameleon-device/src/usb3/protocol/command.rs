use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use crate::usb3::{Error, Result};

#[derive(Debug)]
pub struct CommandPacket<'a> {
    ccd: CommandCcd,
    scd: Box<dyn CommandScd + 'a>,
}

impl<'a> CommandPacket<'a> {
    const PREFIX_MAGIC: u32 = 0x43563355;

    // Magic + CCD length.
    const ACK_HEADER_LENGTH: usize = 4 + 8;

    // Length of pending ack SCD. This SCD can be returned with any command.
    const MINIMUM_ACK_SCD_LENGTH: usize = 4;

    pub fn serialize(&self, mut buf: impl Write) -> Result<()> {
        buf.write_u32::<LE>(Self::PREFIX_MAGIC)?;
        self.ccd.serialize(&mut buf)?;
        self.scd.serialize(&mut buf)?;

        Ok(())
    }

    pub fn cmd_len(&self) -> usize {
        // Magic(4bytes) + ccd + scd
        4 + self.ccd.len() as usize + self.scd.len() as usize
    }

    pub fn request_id(&self) -> u16 {
        self.ccd.request_id
    }

    /// Maximum length of corresponding ack packet.
    pub fn maximum_ack_len(&self) -> Option<usize> {
        let scd_len = self.scd.maximum_ack_scd_len()?;
        let maximum_scd_length = std::cmp::max(scd_len, Self::MINIMUM_ACK_SCD_LENGTH);

        Some(Self::ACK_HEADER_LENGTH + maximum_scd_length)
    }

    fn new(scd: impl CommandScd + 'a, request_id: u16) -> Self {
        let ccd = CommandCcd::new(&scd, request_id);
        let scd = Box::new(scd);
        Self { ccd, scd }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMem {
    pub address: u64,
    pub read_length: u16,
}

impl ReadMem {
    pub fn new(address: u64, read_length: u16) -> Self {
        Self {
            address,
            read_length,
        }
    }

    pub fn finalize(self, request_id: u16) -> CommandPacket<'static> {
        CommandPacket::new(self, request_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteMem<'a> {
    pub address: u64,
    pub data: &'a [u8],
}

impl<'a> WriteMem<'a> {
    pub fn new(address: u64, data: &'a [u8]) -> Self {
        Self { address, data }
    }

    pub fn finalize(self, request_id: u16) -> CommandPacket<'a> {
        CommandPacket::new(self, request_id)
    }

    fn data_len(&self) -> u16 {
        // We validates the data length in constructor.
        // So this conversion never panic.
        self.data.len() as u16
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMemStacked<'a> {
    pub entries: &'a [ReadMem],
}

impl<'a> ReadMemStacked<'a> {
    pub fn new(entries: &'a [ReadMem]) -> Self {
        Self { entries }
    }

    pub fn finalize(self, request_id: u16) -> CommandPacket<'a> {
        CommandPacket::new(self, request_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteMemStacked<'a> {
    pub entries: &'a [WriteMem<'a>],
}

impl<'a> WriteMemStacked<'a> {
    pub fn new(entries: &'a [WriteMem<'a>]) -> Self {
        Self { entries }
    }

    pub fn finalize(self, request_id: u16) -> CommandPacket<'a> {
        CommandPacket::new(self, request_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomCommand<'a> {
    command_id: u16,
    data: &'a [u8],
}

impl<'a> CustomCommand<'a> {
    pub fn new(command_id: u16, data: &'a [u8]) -> Result<Self> {
        if command_id >> 15 == 0 {
            return Err(Error::InvalidPacket(
                "custom command id must set MSB to 1".into(),
            ));
        }

        if command_id & 1 == 1 {
            return Err(Error::InvalidPacket("command id must set LSB to 0".into()));
        }

        Ok(Self { command_id, data })
    }

    pub fn finalize(self, request_id: u16) -> CommandPacket<'a> {
        CommandPacket::new(self, request_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommandCcd {
    flag: CommandFlag,
    scd_kind: ScdKind,
    scd_len: usize,
    request_id: u16,
}

impl CommandCcd {
    fn new(scd: &impl CommandScd, request_id: u16) -> Self {
        Self {
            // Currently USB3 commands always request ack.
            flag: scd.flag(),
            scd_kind: scd.scd_kind(),
            scd_len: scd.len(),
            request_id,
        }
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        self.flag.serialize(buf)?;
        self.scd_kind.serialize(buf)?;

        if self.scd_len > u16::MAX as usize {
            let err = format!(
                "maximum SCD length is limited to {}, but {} is given",
                u16::MAX,
                self.scd_len,
            );
            return Err(Error::InvalidPacket(err.into()));
        }
        buf.write_u16::<LE>(self.scd_len as u16)?;

        buf.write_u16::<LE>(self.request_id)?;
        Ok(())
    }

    const fn len(&self) -> u16 {
        // flags(2bytes) + command_id(2bytes) + scd_len(2bytes) + request_id(2bytes)
        8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CommandFlag {
    RequestAck,
}

impl CommandFlag {
    pub fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        let flag_id = match self {
            Self::RequestAck => 1 << 14,
        };
        Ok(buf.write_u16::<LE>(flag_id)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ScdKind {
    ReadMem,
    WriteMem,
    ReadMemStacked,
    WriteMemStacked,
    Custom(u16),
}

impl ScdKind {
    pub fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        let kind_id = match self {
            Self::ReadMem => 0x0800,
            Self::WriteMem => 0x0802,
            Self::ReadMemStacked => 0x0806,
            Self::WriteMemStacked => 0x0808,
            Self::Custom(id) => *id,
        };
        Ok(buf.write_u16::<LE>(kind_id)?)
    }
}

trait CommandScd: std::fmt::Debug {
    fn flag(&self) -> CommandFlag;

    fn scd_kind(&self) -> ScdKind;

    fn len(&self) -> usize;

    fn serialize(&self, buf: &mut dyn Write) -> Result<()>;

    fn maximum_ack_scd_len(&self) -> Option<usize>;
}

impl CommandScd for ReadMem {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::ReadMem
    }

    fn len(&self) -> usize {
        // Address(8 bytes) + reserved(2bytes) + length(2 bytes)
        12
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        buf.write_u64::<LE>(self.address)?;
        buf.write_u16::<LE>(0)?; // 2bytes reserved.
        buf.write_u16::<LE>(self.read_length)?;
        Ok(())
    }

    fn maximum_ack_scd_len(&self) -> Option<usize> {
        Some(self.read_length as usize)
    }
}

impl<'a> CommandScd for WriteMem<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::WriteMem
    }

    fn len(&self) -> usize {
        // Address(8bytes) + data length.
        8 + self.data.len()
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        buf.write_u64::<LE>(self.address)?;
        buf.write_all(self.data)?;
        Ok(())
    }

    fn maximum_ack_scd_len(&self) -> Option<usize> {
        // Reserved(2bytes)+ length written(2bytes);
        Some(4)
    }
}

impl<'a> CommandScd for ReadMemStacked<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::ReadMemStacked
    }

    fn len(&self) -> usize {
        self.entries.iter().fold(0, |acc, ent| acc + ent.len())
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        for ent in self.entries {
            ent.serialize(buf)?;
        }
        Ok(())
    }

    fn maximum_ack_scd_len(&self) -> Option<usize> {
        todo!();
    }
}

impl<'a> CommandScd for WriteMemStacked<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::WriteMemStacked
    }

    fn len(&self) -> usize {
        // Each entry is composed of [address(8bytes), reserved(2bytes), data_len(2bytes), data(len bytes)]
        self.entries
            .iter()
            .fold(0, |acc, ent| acc + 12 + ent.data.len())
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        for ent in self.entries {
            buf.write_u64::<LE>(ent.address)?;
            buf.write_u16::<LE>(0)?; // 2bytes reserved.
            buf.write_u16::<LE>(ent.data_len())?;
            buf.write_all(ent.data)?;
        }
        Ok(())
    }

    fn maximum_ack_scd_len(&self) -> Option<usize> {
        todo!();
    }
}

impl<'a> CommandScd for CustomCommand<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::Custom(self.command_id)
    }

    fn len(&self) -> usize {
        self.data.len() as usize
    }

    fn serialize(&self, buf: &mut dyn Write) -> Result<()> {
        buf.write_all(self.data)?;
        Ok(())
    }

    fn maximum_ack_scd_len(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER_LEN: u8 = 4 + 8; // Magic + CCD.

    fn serialize_header(command_id: &[u8; 2], scd_len: &[u8; 2], req_id: &[u8; 2]) -> Vec<u8> {
        let mut ccd = vec![];
        ccd.write_u32::<LE>(0x43563355).unwrap(); // Magic.
        ccd.extend(&[0x00, 0x40]); // Packet flag: Request Ack.
        ccd.extend(command_id);
        ccd.extend(scd_len);
        ccd.extend(req_id);
        ccd
    }

    #[test]
    fn test_read_mem_cmd() {
        let command = ReadMem::new(0x0004, 64).finalize(1);
        let scd_len = 12;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header(&[0x00, 0x08], &[scd_len, 0x00], &[0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address.
        expected.extend(vec![0x00, 0x00]); // Reserved.
        expected.extend(vec![64, 0x00]); // Read length.

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_write_mem_cmd() {
        let command = WriteMem::new(0x0004, &[0x01, 0x02, 0x03]).finalize(1);
        let scd_len = 11;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header(&[0x02, 0x08], &[scd_len, 0x00], &[0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address.
        expected.extend(vec![0x01, 0x02, 0x03]); // Data.

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_read_mem_stacked() {
        let mut read_mems = vec![];
        read_mems.push(ReadMem::new(0x0004, 4));
        read_mems.push(ReadMem::new(0x0008, 8));

        let command = ReadMemStacked::new(&read_mems).finalize(1);
        let scd_len = 12 * 2;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header(&[0x06, 0x08], &[12 * 2, 0x00], &[0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address 0.
        expected.extend(vec![0x00, 0x00]); // Reserved 0.
        expected.extend(vec![4, 0x00]); // Read length 0.
        expected.extend(vec![0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address 1.
        expected.extend(vec![0x00, 0x00]); // Reserved 1.
        expected.extend(vec![8, 0x00]); // Read length 1.

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_write_mem_stacked() {
        let mut write_mems = vec![];
        write_mems.push(WriteMem::new(0x0004, &[0x01, 0x02, 0x03, 0x04]));
        write_mems.push(WriteMem::new(0x0008, &[0x11, 0x12, 0x13, 0x14]));

        let command = WriteMemStacked::new(&write_mems).finalize(1);
        let scd_len = (12 + 4) * 2;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header(&[0x08, 0x08], &[scd_len, 0x00], &[0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address 0.
        expected.extend(vec![0x00, 0x00]); // Reserved 0.
        expected.extend(vec![4, 0x00]); // Length data block 0.
        expected.extend(vec![0x01, 0x02, 0x03, 0x04]); // Data block 0.
        expected.extend(vec![0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address 1.
        expected.extend(vec![0x00, 0x00]); // Reserved 1.
        expected.extend(vec![4, 0x00]); // Length data block 1.
        expected.extend(vec![0x11, 0x12, 0x13, 0x14]); // Data block 1.

        assert_eq!(buf, expected);
    }
}
