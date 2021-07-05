/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io;

use cameleon_impl::bytes_io::WriteBytes;

use crate::gige::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandPacket<T> {
    header: CommandHeader,
    command_data: T,
}

impl<T> CommandPacket<T>
where
    T: CommandData,
{
    pub fn new(command_data: T, request_id: u16) -> Self {
        let header = CommandHeader::new(&command_data, request_id);
        Self {
            header,
            command_data,
        }
    }

    pub fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        self.header.serialize(&mut buf)?;
        self.command_data.serialize(&mut buf)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHeader {
    flag: CommandFlag,
    command_kind: CommandKind,
    length: u16,
    request_id: u16,
}

impl CommandHeader {
    pub fn new(command_data: &impl CommandData, request_id: u16) -> Self {
        let flag = command_data.flag();
        let command_kind = command_data.kind();
        let length = command_data.length();
        Self {
            flag,
            command_kind,
            length,
            request_id,
        }
    }

    pub fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        const MAGIC: u8 = 0x42;

        buf.write_bytes_be(MAGIC)?;
        self.flag.serialize(&mut buf)?;
        self.command_kind.serialize(&mut buf)?;
        buf.write_bytes_be(self.length)?;
        buf.write_bytes_be(self.request_id)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    Discovery,
    ReadReg,
    WriteReg,
    ReadMem,
    WriteMem,
    PacketResend,
}

impl CommandKind {
    pub fn serialize(self, mut buf: impl io::Write) -> Result<()> {
        let value: u16 = match self {
            Self::Discovery => 0x0002,
            Self::ReadReg => 0x0080,
            Self::WriteReg => 0x0082,
            Self::ReadMem => 0x0084,
            Self::WriteMem => 0x0086,
            Self::PacketResend => 0x0040,
        };

        buf.write_bytes_be(value)?;
        Ok(())
    }
}

pub trait CommandData: Sized {
    fn flag(&self) -> CommandFlag;

    fn kind(&self) -> CommandKind;

    fn length(&self) -> u16;

    fn serialize(&self, buf: impl io::Write) -> Result<()>;

    fn finalize(self, request_id: u16) -> CommandPacket<Self> {
        CommandPacket::new(self, request_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Discovery {
    bloadcast: bool,
}

impl Discovery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bloadcast(&mut self, is_allow: bool) {
        self.bloadcast = is_allow;
    }
}

impl CommandData for Discovery {
    fn flag(&self) -> CommandFlag {
        let flag = CommandFlag::new().need_ack();
        if self.bloadcast {
            flag.set_bit(3)
        } else {
            flag
        }
    }

    fn kind(&self) -> CommandKind {
        CommandKind::Discovery
    }

    fn length(&self) -> u16 {
        0
    }

    fn serialize(&self, _: impl io::Write) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ReadReg {
    addresses: Vec<u32>,
}

impl ReadReg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, address: u32) -> Result<()> {
        const MAXIMUM_ENTRY_NUMBER: usize = 135;
        if self.addresses.len() >= MAXIMUM_ENTRY_NUMBER {
            return Err(Error::InvalidPacket(
                format!(
                    "a number of entry of `ReadReg` must be smaller or equal than {}",
                    MAXIMUM_ENTRY_NUMBER
                )
                .into(),
            ));
        } else if address % 4 != 0 {
            Err(Error::InvalidPacket(
                "an address of `ReadReg` must be a multiple of 4".into(),
            ))
        } else {
            self.addresses.push(address);
            Ok(())
        }
    }
}

impl CommandData for ReadReg {
    fn flag(&self) -> CommandFlag {
        CommandFlag::new().need_ack()
    }

    fn kind(&self) -> CommandKind {
        CommandKind::ReadReg
    }

    fn length(&self) -> u16 {
        (self.addresses.len() * std::mem::size_of::<u32>()) as u16
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        for address in &self.addresses {
            buf.write_bytes_be(*address)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WriteRegEntry {
    address: u32,
    data: u32,
}

impl WriteRegEntry {
    pub fn new(address: u32, data: u32) -> Result<Self> {
        if address % 4 == 0 {
            Ok(Self { address, data })
        } else {
            Err(Error::InvalidPacket(
                "an address of `WriteReg` must be a multiple of 4".into(),
            ))
        }
    }

    const fn length() -> u16 {
        64
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        buf.write_bytes_be(self.address)?;
        buf.write_bytes_be(self.data)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteReg {
    entries: Vec<WriteRegEntry>,
    need_ack: bool,
}

impl Default for WriteReg {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            need_ack: true,
        }
    }
}

impl WriteReg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, entry: WriteRegEntry) -> Result<()> {
        const MAXIMUM_ENTRY_NUMBER: usize = 67;
        if self.entries.len() >= MAXIMUM_ENTRY_NUMBER {
            Err(Error::InvalidPacket(
                format!(
                    "a number of entry of `WriteReg` must be smaller or equal than {}",
                    MAXIMUM_ENTRY_NUMBER
                )
                .into(),
            ))
        } else {
            self.entries.push(entry);
            Ok(())
        }
    }

    pub fn set_need_ack(&mut self, need_ack: bool) {
        self.need_ack = need_ack
    }
}

impl CommandData for WriteReg {
    fn flag(&self) -> CommandFlag {
        if self.need_ack {
            CommandFlag::new().need_ack()
        } else {
            CommandFlag::new()
        }
    }

    fn kind(&self) -> CommandKind {
        CommandKind::WriteReg
    }

    fn length(&self) -> u16 {
        self.entries.len() as u16 * WriteRegEntry::length()
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        for ent in &self.entries {
            ent.serialize(&mut buf)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMem {
    address: u32,
    length: u16,
}

impl ReadMem {
    pub fn new(address: u32, length: u16) -> Result<Self> {
        const MAXIMUM_READ_LENGTH: u16 = 536;

        if address % 4 != 0 && length % 4 != 0 {
            Err(Error::InvalidPacket(
                "address and length fields of `ReadMem` command must be a multiple of 4".into(),
            ))
        } else if length > MAXIMUM_READ_LENGTH {
            Err(Error::InvalidPacket(
                format!(
                    "length must be smaller or equal than {}",
                    MAXIMUM_READ_LENGTH
                )
                .into(),
            ))
        } else {
            Ok(Self { address, length })
        }
    }
}

impl CommandData for ReadMem {
    fn flag(&self) -> CommandFlag {
        CommandFlag::new()
    }

    fn kind(&self) -> CommandKind {
        CommandKind::ReadMem
    }

    fn length(&self) -> u16 {
        6
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        buf.write_bytes_be(self.address)?;
        buf.write_bytes_be(self.length)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteMem<'a> {
    address: u32,
    data: &'a [u8],
    need_ack: bool,
}

impl<'a> WriteMem<'a> {
    pub fn new(address: u32, data: &'a [u8]) -> Result<Self> {
        const MAXIMUM_DATA_LEN: usize = 536;
        if address % 4 != 0 {
            Err(Error::InvalidPacket(
                "an address of `WriteMem` command must be a multiple of 4".into(),
            ))
        } else if data.len() > MAXIMUM_DATA_LEN {
            Err(Error::InvalidPacket(
                format!(
                    "a data length of `WriteMem` command must be smaller or equal than {}",
                    MAXIMUM_DATA_LEN
                )
                .into(),
            ))
        } else {
            Ok(Self {
                address,
                data,
                need_ack: true,
            })
        }
    }

    pub fn set_need_ack(&mut self, need_ack: bool) {
        self.need_ack = need_ack
    }
}

impl<'a> CommandData for WriteMem<'a> {
    fn flag(&self) -> CommandFlag {
        if self.need_ack {
            CommandFlag::new().need_ack()
        } else {
            CommandFlag::new()
        }
    }

    fn kind(&self) -> CommandKind {
        CommandKind::WriteMem
    }

    fn length(&self) -> u16 {
        32 + self.data.len() as u16
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        buf.write_bytes_be(self.address)?;
        buf.write_all(self.data)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketResend {
    is_extended_id: bool,
    stream_channel_index: u16,
    first_packet_id: u32,
    last_packet_id: u32,
    block_id: u64,
}

impl PacketResend {
    pub fn with_extended_id(
        stream_channel_index: u16,
        first_packet_id: u32,
        last_packet_id: u32,
        block_id: u64,
    ) -> Self {
        Self {
            is_extended_id: true,
            stream_channel_index,
            first_packet_id,
            last_packet_id,
            block_id,
        }
    }

    pub fn with_unextended_id(
        stream_channel_index: u16,
        first_packet_id: u32,
        last_packet_id: u32,
        block_id: u16,
    ) -> Result<Self> {
        const UNEXTENDED_MAXIMUM_PACKET_ID: u32 = 16777215; // 2 ** 24 - 1
        if first_packet_id > UNEXTENDED_MAXIMUM_PACKET_ID
            || last_packet_id > UNEXTENDED_MAXIMUM_PACKET_ID
        {
            Err(Error::InvalidPacket(
                format!(
                    "a maximum packet id with unextedned_id_mode is {}",
                    UNEXTENDED_MAXIMUM_PACKET_ID
                )
                .into(),
            ))
        } else {
            Ok(Self {
                is_extended_id: true,
                stream_channel_index,
                first_packet_id,
                last_packet_id,
                block_id: block_id as u64,
            })
        }
    }
}

impl CommandData for PacketResend {
    fn flag(&self) -> CommandFlag {
        if self.is_extended_id {
            CommandFlag::new().set_bit(3)
        } else {
            CommandFlag::new()
        }
    }

    fn kind(&self) -> CommandKind {
        CommandKind::PacketResend
    }

    fn length(&self) -> u16 {
        // `stream_channel_index` + `block_id/reserved` + `first_packet_id` + `last_packet_id`.
        const LENGTH: u16 = 16 + 16 + 32 + 32;
        if self.is_extended_id {
            // 64bit block_id.
            LENGTH + 64
        } else {
            LENGTH
        }
    }

    fn serialize(&self, mut buf: impl io::Write) -> Result<()> {
        buf.write_bytes_be(self.stream_channel_index)?;
        if self.is_extended_id {
            buf.write_bytes_be(0_u16)?;
        } else {
            buf.write_bytes_be(self.block_id as u16)?;
        }
        buf.write_bytes_be(self.first_packet_id)?;
        buf.write_bytes_be(self.last_packet_id)?;
        if self.is_extended_id {
            buf.write_bytes_be(self.block_id)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CommandFlag(u8);

impl CommandFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bit(self, pos: u8) -> Self {
        debug_assert!(pos <= 8);
        Self(self.0 | 1_u8 << pos)
    }

    pub fn clear_bit(self, pos: u8) -> Self {
        debug_assert!(pos <= 8);
        Self(self.0 & !(1_u8 << pos))
    }

    pub fn need_ack(self) -> Self {
        self.set_bit(7)
    }

    pub fn serialize(self, mut buf: impl io::Write) -> Result<()> {
        buf.write_bytes_be(self.0)?;
        Ok(())
    }
}
