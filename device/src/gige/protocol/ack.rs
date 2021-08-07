/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    convert::TryInto,
    io::{self, Read, Seek},
    net::Ipv4Addr,
    time,
};

use cameleon_impl::bytes_io::ReadBytes;
use semver::Version;

use crate::gige::{
    register_map::{DeviceMode, NicCapability, NicConfiguration},
    Error, Result,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AckPacket<'a> {
    header: Header,
    raw_ack_data: &'a [u8],
}
impl<'a> AckPacket<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self> {
        let mut cursor = io::Cursor::new(buf);
        let header = Header::parse(&mut cursor)?;

        if buf.len() < header.length as usize {
            return Err(Error::InvalidPacket(
                "ack data length is smaller than specified length in header".into(),
            ));
        }

        let raw_ack_data = &cursor.get_ref()[cursor.position() as usize..];
        Ok(Self {
            header,
            raw_ack_data,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn ack_kind(&self) -> AckKind {
        self.header.ack_kind
    }

    pub fn ack_data_as<T: ParseAckData<'a>>(&self) -> Result<T> {
        T::parse(self.raw_ack_data, &self.header)
    }

    pub fn raw_ack_data(&self) -> &[u8] {
        self.raw_ack_data
    }

    pub fn request_id(&self) -> u16 {
        self.header.request_id
    }

    pub fn status(&self) -> Status {
        self.header.status
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    status: Status,
    ack_kind: AckKind,
    length: u16,
    request_id: u16,
}

impl Header {
    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let status = Status::parse(cursor)?;
        let ack_kind = AckKind::parse(cursor)?;
        let length = cursor.read_bytes_be()?;
        let request_id = cursor.read_bytes_be()?;
        Ok(Self {
            status,
            ack_kind,
            length,
            request_id,
        })
    }
}

pub trait ParseAckData<'a>: Sized {
    fn parse(raw_data: &'a [u8], header: &Header) -> Result<Self>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    code: u16,
    kind: StatusKind,
}

impl Status {
    pub fn is_success(self) -> bool {
        matches!(self.kind, StatusKind::Success | StatusKind::PacketResend)
    }

    pub fn code(self) -> u16 {
        self.code
    }

    pub fn kind(self) -> StatusKind {
        self.kind
    }

    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let code: u16 = cursor.read_bytes_be()?;
        let kind = match code {
            0x0000 => StatusKind::Success,
            0x0100 => StatusKind::PacketResend,
            0x8001 => StatusKind::NotImplemented,
            0x8002 => StatusKind::InvalidParameter,
            0x8003 => StatusKind::InvalidAdderess,
            0x8004 => StatusKind::WriteProtect,
            0x8005 => StatusKind::BadAlignment,
            0x8006 => StatusKind::AccessDenied,
            0x8007 => StatusKind::Busy,
            0x8008 => StatusKind::LocalProblem,
            0x8009 => StatusKind::MessageMismatch,
            0x800a => StatusKind::InvalidProtocol,
            0x800b => StatusKind::NoMessage,
            0x800c => StatusKind::PacketUnavailable,
            0x800d => StatusKind::DataOverrun,
            0x800e => StatusKind::InvalidHeader,
            0x800f => StatusKind::WrongConfig,
            0x8010 => StatusKind::NotYetAvailable,
            0x8011 => StatusKind::CurrentAndPrevPacketRemoved,
            0x8012 => StatusKind::CurrentPacketRemoved,
            0x8013 => StatusKind::NoReferenceTime,
            0x8014 => StatusKind::TemporarilyUnavailable,
            0x8015 => StatusKind::Overflow,
            0x8016 => StatusKind::ActionLate,
            0x8fff => StatusKind::GenericError,
            _ => {
                return Err(Error::InvalidPacket(
                    format! {"invalid gige ack status code {:#X}", code}.into(),
                ));
            }
        };
        Ok(Self { code, kind })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    PacketResend,
    NotImplemented,
    InvalidParameter,
    InvalidAdderess,
    WriteProtect,
    BadAlignment,
    AccessDenied,
    Busy,
    LocalProblem,
    MessageMismatch,
    InvalidProtocol,
    NoMessage,
    PacketUnavailable,
    DataOverrun,
    InvalidHeader,
    WrongConfig,
    NotYetAvailable,
    CurrentAndPrevPacketRemoved,
    CurrentPacketRemoved,
    NoReferenceTime,
    TemporarilyUnavailable,
    Overflow,
    ActionLate,
    GenericError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckKind {
    Discovery,
    ReadReg,
    WriteReg,
    ReadMem,
    WriteMem,
    PacketResend,
    Pending,
}

impl AckKind {
    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let id: u16 = cursor.read_bytes_be()?;
        match id {
            0x0003 => Ok(AckKind::Discovery),
            0x0081 => Ok(AckKind::ReadReg),
            0x0083 => Ok(AckKind::WriteReg),
            0x0085 => Ok(AckKind::ReadMem),
            0x0087 => Ok(AckKind::WriteMem),
            0x0089 => Ok(AckKind::Pending),
            _ => Err(Error::InvalidPacket(
                format!("unknown ack kind id {:#X}", id).into(),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovery {
    pub gige_version: semver::Version,
    pub device_mode: DeviceMode,
    pub mac_addr: [u8; 6],
    pub nic_capability: NicCapability,
    pub nic_configuration: NicConfiguration,
    pub ip: Ipv4Addr,
    pub subnet_mask: [u8; 4],
    pub default_gateway: Ipv4Addr,
    pub manufacturer_name: String,
    pub model_name: String,
    pub device_version: String,
    pub manufacturer_specific_info: String,
    pub serial_number: String,
    pub user_defined_name: String,
}

impl Discovery {
    fn read_string(cursor: &mut io::Cursor<&[u8]>, maximum_string_len: usize) -> Result<String> {
        let str_start = cursor.position() as usize;
        let str_end = maximum_string_len + str_start;
        let inner = cursor.get_ref();
        if inner.len() < str_end {
            return Err(Error::InvalidPacket(
                format!(
                    "size of received discovery ack is too small: size {}",
                    inner.len(),
                )
                .into(),
            ));
        }

        let str_end = inner[str_start..str_end]
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(str_end);
        let s = String::from_utf8_lossy(&inner[str_start..str_start + str_end]).to_string();
        cursor.seek(io::SeekFrom::Current(maximum_string_len as i64))?;
        Ok(s)
    }
}

impl<'a> ParseAckData<'a> for Discovery {
    fn parse(raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::Discovery {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `Discovery` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }

        let mut cursor = io::Cursor::new(raw_data);
        let version_major: u16 = cursor.read_bytes_be()?;
        let version_minor: u16 = cursor.read_bytes_be()?;
        let gige_version = Version::new(version_major as u64, version_minor as u64, 0);
        let device_mode = DeviceMode::from_raw(cursor.read_bytes_be()?);
        cursor.seek(io::SeekFrom::Current(2))?;
        let mut mac_addr = [0; 6];
        cursor.read_exact(&mut mac_addr[..])?;
        let nic_capability = NicCapability::from_raw(cursor.read_bytes_be()?);
        let nic_configuration = NicConfiguration::from_raw(cursor.read_bytes_be()?);
        cursor.seek(io::SeekFrom::Current(12))?;
        let ip = cursor.read_bytes_be()?;
        cursor.seek(io::SeekFrom::Current(12))?;
        let mut subnet_mask = [0; 4];
        cursor.read_exact(&mut subnet_mask)?;
        cursor.seek(io::SeekFrom::Current(12))?;
        let default_gateway = cursor.read_bytes_be()?;

        let manufacturer_name = Self::read_string(&mut cursor, 32)?;
        let model_name = Self::read_string(&mut cursor, 32)?;
        let device_version = Self::read_string(&mut cursor, 32)?;
        let manufacturer_specific_info = Self::read_string(&mut cursor, 48)?;
        let serial_number = Self::read_string(&mut cursor, 16)?;
        let user_defined_name = Self::read_string(&mut cursor, 16)?;

        Ok(Self {
            gige_version,
            device_mode,
            mac_addr,
            nic_capability,
            nic_configuration,
            ip,
            subnet_mask,
            default_gateway,
            manufacturer_name,
            model_name,
            device_version,
            manufacturer_specific_info,
            serial_number,
            user_defined_name,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadReg<'a> {
    entry_num: u16,
    reg_data: &'a [u8],
}

impl<'a> ReadReg<'a> {
    pub fn entry_num(self) -> u16 {
        self.entry_num
    }

    pub fn iter(&self) -> ReadRegIter {
        ReadRegIter {
            entry_num: self.entry_num,
            current_entry: 0,
            reg_data: self.reg_data,
        }
    }
}

impl<'a> ParseAckData<'a> for ReadReg<'a> {
    fn parse(raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::ReadReg {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `ReadReg` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }

        let length = header.length;
        if length % 4 != 0 {
            return Err(Error::InvalidPacket(
                ("data of `ReadReg` ack must be a multiple of 4").into(),
            ));
        }
        let entry_num = length / 4;
        Ok(Self {
            entry_num,
            reg_data: raw_data,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadRegIter<'a> {
    entry_num: u16,
    current_entry: u16,
    reg_data: &'a [u8],
}

impl<'a> IntoIterator for ReadReg<'a> {
    type Item = &'a [u8; 4];
    type IntoIter = ReadRegIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ReadRegIter {
            entry_num: self.entry_num,
            current_entry: 0,
            reg_data: self.reg_data,
        }
    }
}

impl<'a> Iterator for ReadRegIter<'a> {
    type Item = &'a [u8; 4];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry > self.entry_num {
            None
        } else {
            let current_index = self.current_entry as usize * 4;
            let item = Some(
                (&self.reg_data[current_index..current_index + 4])
                    .try_into()
                    .unwrap(),
            );
            self.current_entry += 1;
            item
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteReg {
    entry_num: u16,
}

impl WriteReg {
    pub fn entry_num(self) -> u16 {
        self.entry_num
    }
}

impl<'a> ParseAckData<'a> for WriteReg {
    fn parse(raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::WriteReg {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `WriteReg` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }

        let entry_num = (&raw_data[2..4]).read_bytes_be()?;
        Ok(Self { entry_num })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadMem<'a> {
    address: u32,
    data: &'a [u8],
}

impl<'a> ReadMem<'a> {
    pub fn address(&self) -> u32 {
        self.address
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> ParseAckData<'a> for ReadMem<'a> {
    fn parse(mut raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::ReadMem {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `ReadMem` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }
        let address = raw_data.read_bytes_be()?;

        let data_length = header.length as usize - 4;
        Ok(Self {
            address,
            data: &raw_data[..data_length],
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteMem {
    address: u32,
    num_bytes: u16,
}

impl WriteMem {
    pub fn address(self) -> u32 {
        self.address
    }

    pub fn num_bytes(self) -> u16 {
        self.num_bytes
    }
}

impl<'a> ParseAckData<'a> for WriteMem {
    fn parse(mut raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::WriteMem {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `WriteMem` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }
        let address = raw_data.read_bytes_be()?;
        let num_bytes = raw_data.read_bytes_be()?;
        Ok(Self { address, num_bytes })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    waiting_time: u16,
}

impl Pending {
    pub fn waiting_time(self) -> time::Duration {
        time::Duration::from_millis(self.waiting_time as u64)
    }
}

impl<'a> ParseAckData<'a> for Pending {
    fn parse(raw_data: &'a [u8], header: &Header) -> Result<Self> {
        if header.ack_kind != AckKind::Pending {
            return Err(Error::InvalidPacket(
                format!(
                    "invalid ack kind: expected `Pending` but {:?}",
                    header.ack_kind
                )
                .into(),
            ));
        }
        let waiting_time = (&raw_data[2..4]).read_bytes_be()?;
        Ok(Self { waiting_time })
    }
}
