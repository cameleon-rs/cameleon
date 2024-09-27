/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryInto, io::Write};

use cameleon_impl::bytes_io::WriteBytes;

use crate::u3v::{Error, Result};

#[derive(Debug)]
pub struct CommandPacket<T> {
    ccd: CommandCcd,
    scd: T,
}

impl<T> CommandPacket<T>
where
    T: CommandScd,
{
    const PREFIX_MAGIC: u32 = 0x4356_3355;

    // Magic + CCD length.
    const ACK_HEADER_LENGTH: usize = 4 + 8;

    // Length of pending ack SCD. This SCD can be returned with any command.
    const MINIMUM_ACK_SCD_LENGTH: u16 = 4;

    pub fn serialize(&self, mut buf: impl Write) -> Result<()> {
        buf.write_bytes_le(Self::PREFIX_MAGIC)?;
        self.ccd.serialize(&mut buf)?;
        self.scd.serialize(&mut buf)?;

        Ok(())
    }

    pub fn ccd(&self) -> &CommandCcd {
        &self.ccd
    }

    pub fn scd(&self) -> &T {
        &self.scd
    }

    pub fn cmd_len(&self) -> usize {
        // Magic(4bytes) + ccd + scd
        4 + CommandCcd::len() as usize + self.scd.scd_len() as usize
    }

    pub fn request_id(&self) -> u16 {
        self.ccd.request_id
    }

    /// Maximum length of corresponding ack packet.
    pub fn maximum_ack_len(&self) -> usize {
        let scd_len = self.scd.ack_scd_len();
        let maximum_scd_length = std::cmp::max(scd_len, Self::MINIMUM_ACK_SCD_LENGTH) as usize;

        Self::ACK_HEADER_LENGTH + maximum_scd_length
    }

    pub fn new(scd: T, request_id: u16) -> Self {
        let ccd = CommandCcd::from_scd(&scd, request_id);
        Self { ccd, scd }
    }

    fn header_len() -> usize {
        4 + CommandCcd::len() as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMem {
    pub(crate) address: u64,
    pub(crate) read_length: u16,
}

pub struct ReadMemChunks {
    address: u64,
    read_length: u16,
    maximum_read_length: usize,
}

impl std::iter::Iterator for ReadMemChunks {
    type Item = ReadMem;

    fn next(&mut self) -> Option<ReadMem> {
        if self.read_length == 0 {
            return None;
        }

        if self.read_length as usize > self.maximum_read_length {
            let next_item = ReadMem::new(self.address, self.maximum_read_length as u16);
            self.read_length -= self.maximum_read_length as u16;
            self.address += self.maximum_read_length as u64;
            Some(next_item)
        } else {
            let next_item = ReadMem::new(self.address, self.read_length);
            self.read_length = 0;
            Some(next_item)
        }
    }
}

impl ReadMem {
    #[must_use]
    pub fn new(address: u64, read_length: u16) -> Self {
        Self {
            address,
            read_length,
        }
    }

    /// Split into multiple [`ReadMem`] chunks so that all corresponding ack length fit into `ack_len`.
    pub fn chunks(&self, ack_len: usize) -> Result<ReadMemChunks> {
        let ack_header_length = CommandPacket::<ReadMem>::ACK_HEADER_LENGTH;
        if ack_len <= ack_header_length {
            let msg = format!(
                "ack length must be larger than {}",
                CommandPacket::<ReadMem>::ACK_HEADER_LENGTH
            );
            return Err(Error::InvalidPacket(msg.into()));
        };
        let maximum_read_length = ack_len - ack_header_length;

        Ok(ReadMemChunks {
            address: self.address,
            read_length: self.read_length,
            maximum_read_length,
        })
    }

    /// Returns maximum read length that corresponding ack length fit into `maximum_ack_len`.
    pub fn maximum_read_length(maximum_ack_len: usize) -> u16 {
        (maximum_ack_len - CommandPacket::<ReadMem>::ACK_HEADER_LENGTH)
            .try_into()
            .unwrap_or(u16::MAX)
    }

    #[must_use]
    pub fn read_length(&self) -> u16 {
        self.read_length
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteMem<'a> {
    pub(crate) address: u64,
    pub(crate) data: &'a [u8],
    data_len: u16,
    len: u16,
}

pub struct WriteMemChunks<'a> {
    address: u64,
    data: &'a [u8],
    data_idx: usize,
    maximum_data_len: usize,
}

impl<'a> std::iter::Iterator for WriteMemChunks<'a> {
    type Item = WriteMem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data_idx == self.data.len() {
            return None;
        }

        if self.data_idx + self.maximum_data_len < self.data.len() {
            let next_item = WriteMem::new(
                self.address,
                &self.data[self.data_idx..self.data_idx + self.maximum_data_len],
            )
            .unwrap();
            self.address += self.maximum_data_len as u64;
            self.data_idx += self.maximum_data_len;
            Some(next_item)
        } else {
            let next_item = WriteMem::new(self.address, &self.data[self.data_idx..]).unwrap();
            self.data_idx = self.data.len();
            Some(next_item)
        }
    }
}

impl<'a> WriteMem<'a> {
    pub fn new(address: u64, data: &'a [u8]) -> Result<Self> {
        let data_len = into_scd_len(data.len())?;
        let len = into_scd_len(data.len() + 8)?;

        Ok(Self {
            address,
            data,
            data_len,
            len,
        })
    }

    /// Data length of the
    #[must_use]
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// Split into multiple [`WriteMem`] chunks so that all commands resulting from chunks fit into `cmd_len`.
    pub fn chunks(&self, cmd_len: usize) -> Result<WriteMemChunks<'a>> {
        let cmd_header_len = CommandPacket::<WriteMem>::header_len() + 8;
        if cmd_len <= cmd_header_len {
            let msg = format!(
                "cmd_len must be larger than {}",
                CommandPacket::<WriteMem>::header_len() + 8
            );
            return Err(Error::InvalidPacket(msg.into()));
        };
        let maximum_data_len = cmd_len - cmd_header_len;

        Ok(WriteMemChunks {
            address: self.address,
            data: self.data,
            data_idx: 0,
            maximum_data_len,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMemStacked {
    pub(crate) entries: Vec<ReadMem>,
    len: u16,
    ack_scd_len: u16,
}

impl ReadMemStacked {
    pub fn new(entries: Vec<ReadMem>) -> Result<Self> {
        let len = Self::len(&entries)?;
        let ack_scd_len = Self::ack_scd_len(&entries)?;

        Ok(Self {
            entries,
            len,
            ack_scd_len,
        })
    }

    fn len(regs: &[ReadMem]) -> Result<u16> {
        let len = regs.iter().fold(0, |acc, reg| acc + reg.scd_len() as usize);
        into_scd_len(len)
    }

    fn ack_scd_len(entries: &[ReadMem]) -> Result<u16> {
        let mut acc: u16 = 0;
        for ent in entries {
            acc = acc.checked_add(ent.read_length).ok_or_else(|| {
                Error::InvalidPacket("total read length must be less than u16::MAX".into())
            })?;
        }

        Ok(acc)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteMemStacked<'a> {
    pub(crate) entries: Vec<WriteMem<'a>>,
    len: u16,
    ack_scd_len: u16,
}

impl<'a> WriteMemStacked<'a> {
    pub fn new(entries: Vec<WriteMem<'a>>) -> Result<Self> {
        let len = Self::len(&entries)?;
        let ack_scd_len = entries.len() as u16 * 4;
        Ok(Self {
            entries,
            len,
            ack_scd_len,
        })
    }

    fn len(entries: &[WriteMem<'a>]) -> Result<u16> {
        let len = entries
            .iter()
            .fold(0, |acc, cmd| acc + 12 + cmd.data_len as usize);
        into_scd_len(len)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandCcd {
    flag: CommandFlag,
    scd_kind: ScdKind,
    scd_len: u16,
    request_id: u16,
}

impl CommandCcd {
    #[must_use]
    pub fn flag(&self) -> CommandFlag {
        self.flag
    }

    #[must_use]
    pub fn scd_kind(&self) -> ScdKind {
        self.scd_kind
    }

    #[must_use]
    pub fn scd_len(&self) -> u16 {
        self.scd_len
    }

    #[must_use]
    pub fn request_id(&self) -> u16 {
        self.request_id
    }

    pub(crate) fn new(flag: CommandFlag, scd_kind: ScdKind, scd_len: u16, request_id: u16) -> Self {
        Self {
            flag,
            scd_kind,
            scd_len,
            request_id,
        }
    }

    fn from_scd(scd: &impl CommandScd, request_id: u16) -> Self {
        Self::new(scd.flag(), scd.scd_kind(), scd.scd_len(), request_id)
    }

    fn serialize(&self, mut buf: impl Write) -> Result<()> {
        self.flag.serialize(&mut buf)?;
        self.scd_kind.serialize(&mut buf)?;
        buf.write_bytes_le(self.scd_len)?;

        buf.write_bytes_le(self.request_id)?;
        Ok(())
    }

    #[must_use]
    pub const fn len() -> u16 {
        // flags(2bytes) + command_id(2bytes) + scd_len(2bytes) + request_id(2bytes)
        8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandFlag {
    RequestAck,
    CommandResend,
}

impl CommandFlag {
    fn serialize(self, mut buf: impl Write) -> Result<()> {
        let flag_id: u16 = match self {
            Self::RequestAck => 1 << 14,
            Self::CommandResend => 1 << 15,
        };

        buf.write_bytes_le(flag_id)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScdKind {
    ReadMem,
    WriteMem,
    ReadMemStacked,
    WriteMemStacked,
}

impl ScdKind {
    fn serialize(self, mut buf: impl Write) -> Result<()> {
        let kind_id: u16 = match self {
            Self::ReadMem => 0x0800,
            Self::WriteMem => 0x0802,
            Self::ReadMemStacked => 0x0806,
            Self::WriteMemStacked => 0x0808,
        };

        buf.write_bytes_le(kind_id)?;
        Ok(())
    }
}

pub trait CommandScd: std::fmt::Debug + Sized {
    fn flag(&self) -> CommandFlag;

    fn scd_kind(&self) -> ScdKind;

    fn scd_len(&self) -> u16;

    fn serialize(&self, buf: impl Write) -> Result<()>;

    fn ack_scd_len(&self) -> u16;

    fn finalize(self, request_id: u16) -> CommandPacket<Self> {
        CommandPacket::new(self, request_id)
    }
}

impl CommandScd for ReadMem {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::ReadMem
    }

    fn scd_len(&self) -> u16 {
        // Address(8 bytes) + reserved(2bytes) + length(2 bytes)
        12
    }

    fn serialize(&self, mut buf: impl Write) -> Result<()> {
        buf.write_bytes_le(self.address)?;
        buf.write_bytes_le(0_u16)?; // 2bytes reserved.
        buf.write_bytes_le(self.read_length)?;
        Ok(())
    }

    fn ack_scd_len(&self) -> u16 {
        self.read_length
    }
}

impl<'a> CommandScd for WriteMem<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::WriteMem
    }

    fn scd_len(&self) -> u16 {
        self.len
    }

    fn serialize(&self, mut buf: impl Write) -> Result<()> {
        buf.write_bytes_le(self.address)?;
        buf.write_all(self.data)?;
        Ok(())
    }

    fn ack_scd_len(&self) -> u16 {
        // Reserved(2bytes)+ length written(2bytes);
        4
    }
}

impl CommandScd for ReadMemStacked {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::ReadMemStacked
    }

    fn scd_len(&self) -> u16 {
        self.len
    }

    fn serialize(&self, mut buf: impl Write) -> Result<()> {
        for ent in &self.entries {
            ent.serialize(&mut buf)?;
        }
        Ok(())
    }

    fn ack_scd_len(&self) -> u16 {
        self.ack_scd_len
    }
}

impl<'a> CommandScd for WriteMemStacked<'a> {
    fn flag(&self) -> CommandFlag {
        CommandFlag::RequestAck
    }

    fn scd_kind(&self) -> ScdKind {
        ScdKind::WriteMemStacked
    }

    fn scd_len(&self) -> u16 {
        // Each register is composed of [address(8bytes), reserved(2bytes), data_len(2bytes), data(len bytes)]
        self.len
    }

    fn serialize(&self, mut buf: impl Write) -> Result<()> {
        for ent in &self.entries {
            buf.write_bytes_le(ent.address)?;
            buf.write_bytes_le(0_u16)?; // 2bytes reserved.
            buf.write_bytes_le(ent.data_len)?;
            buf.write_all(ent.data)?;
        }
        Ok(())
    }

    fn ack_scd_len(&self) -> u16 {
        self.ack_scd_len
    }
}

fn into_scd_len(len: usize) -> Result<u16> {
    len.try_into()
        .map_err(|_| Error::InvalidPacket("scd length must be less than u16::MAX".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER_LEN: u8 = 4 + 8; // Magic + CCD.

    fn serialize_header(command_id: [u8; 2], scd_len: [u8; 2], req_id: [u8; 2]) -> Vec<u8> {
        let mut ccd = vec![];
        ccd.write_bytes_le(0x4356_3355_u32).unwrap(); // Magic.
        ccd.extend([0x00, 0x40]); // Packet flag: Request Ack.
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
        let mut expected = serialize_header([0x00, 0x08], [scd_len, 0x00], [0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address.
        expected.extend(vec![0x00, 0x00]); // Reserved.
        expected.extend(vec![64, 0x00]); // Read length.

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_write_mem_cmd() {
        let command = WriteMem::new(0x0004, &[0x01, 0x02, 0x03])
            .unwrap()
            .finalize(1);
        let scd_len = 11;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header([0x02, 0x08], [scd_len, 0x00], [0x01, 0x00]);
        expected.extend(vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Address.
        expected.extend(vec![0x01, 0x02, 0x03]); // Data.

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_read_mem_stacked() {
        let read_mems = vec![ReadMem::new(0x0004, 4), ReadMem::new(0x0008, 8)];
        let command = ReadMemStacked::new(read_mems).unwrap().finalize(1);
        let scd_len = 12 * 2;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header([0x06, 0x08], [12 * 2, 0x00], [0x01, 0x00]);
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
        let write_mems = vec![
            WriteMem::new(0x0004, &[0x01, 0x02, 0x03, 0x04]).unwrap(),
            WriteMem::new(0x0008, &[0x11, 0x12, 0x13, 0x14]).unwrap(),
        ];

        let command = WriteMemStacked::new(write_mems).unwrap().finalize(1);
        let scd_len = (12 + 4) * 2;

        assert_eq!(command.cmd_len(), (HEADER_LEN + scd_len).into());
        assert_eq!(command.request_id(), 1);

        let mut buf = vec![];
        command.serialize(&mut buf).unwrap();
        let mut expected = serialize_header([0x08, 0x08], [scd_len, 0x00], [0x01, 0x00]);
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

    #[test]
    fn test_read_mem_chunks() {
        let read_mem = ReadMem::new(0, 128);
        let chunks: Vec<_> = read_mem.chunks(24).unwrap().collect();

        let mut expected_addr = 0;
        let mut read_len = 0;
        for (i, chunk) in chunks.iter().enumerate().take(chunks.len() - 1) {
            assert_eq!(chunk.address, expected_addr);
            expected_addr += u64::from(chunk.read_length);
            read_len += chunk.read_length;
            assert_eq!(chunk.clone().finalize(i as u16).maximum_ack_len(), 24);
        }

        let last_chunk = chunks.last().unwrap();
        assert_eq!(last_chunk.address, expected_addr);
        assert_eq!(last_chunk.read_length + read_len, read_mem.read_length);
    }

    #[test]
    fn test_write_mem_chunks() {
        let data = vec![0; 128];
        let write_mem = WriteMem::new(0, &data).unwrap();

        let mut expected_addr = 0;
        let mut sent_data_len = 0;

        let chunks: Vec<_> = write_mem.chunks(24).unwrap().collect();

        for (i, chunk) in chunks.iter().enumerate().take(chunks.len() - 1) {
            assert_eq!(chunk.address, expected_addr);
            expected_addr += u64::from(chunk.data_len);
            sent_data_len += chunk.data_len;

            assert_eq!(chunk.clone().finalize(i as u16).cmd_len(), 24);
        }

        let last_chunk = chunks.last().unwrap();
        assert_eq!(last_chunk.address, expected_addr);
        assert_eq!(last_chunk.data_len, data.len() as u16 - sent_data_len);
    }
}
