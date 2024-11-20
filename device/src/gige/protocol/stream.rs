// TODO:
#![allow(unused)]
use std::{convert::TryInto, io};

use cameleon_impl::{bit_op::BitOp, bytes_io::ReadBytes};

use crate::{
    gige::{Error, Result},
    PixelFormat,
};

use super::PacketStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub status: PacketStatus,
    pub ei_flag: bool,
    pub packet_type: PacketType,
    pub block_id: u64,
    pub packet_id: u32,
    pub stream_flag: StreamFlag,
}

impl PacketHeader {
    pub fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let status = PacketStatus::parse(cursor)?;
        let bid_sflag: u16 = cursor.read_bytes_be()?;
        let ei_ptype_pid: u32 = cursor.read_bytes_be()?;
        let ei_flag = (ei_ptype_pid >> 31) == 1;
        let packet_type = PacketType::parse((ei_ptype_pid >> 24) as u8)?;

        let (block_id, packet_id, stream_flag) = if ei_flag {
            (
                cursor.read_bytes_be()?,
                cursor.read_bytes_be()?,
                StreamFlag(bid_sflag),
            )
        } else {
            (bid_sflag as u64, ei_ptype_pid & 0xffff_ff, StreamFlag(0))
        };

        Ok(Self {
            status,
            ei_flag,
            packet_type,
            block_id,
            packet_id,
            stream_flag,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Leader,
    Trailer,
    GenericPayload,
    H264Payload,
    MultiZonePayload,
}

impl PacketType {
    fn parse(raw: u8) -> Result<Self> {
        Ok(match raw & 0b1111 {
            1 => Self::Leader,
            2 => Self::Trailer,
            3 => Self::GenericPayload,
            5 => Self::H264Payload,
            6 => Self::MultiZonePayload,
            other => {
                return Err(Error::InvalidData(
                    format!("invalid GVSP packet type: {}", other).into(),
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamFlag(u16);

impl StreamFlag {
    pub fn is_resend_range_error(self) -> bool {
        self.0.is_set(13)
    }

    pub fn is_previous_block_dropped(self) -> bool {
        self.0.is_set(14)
    }

    pub fn is_packet_resend(self) -> bool {
        self.0.is_set(15)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadType(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadTypeKind {
    Image,
    RawData,
    File,
    ChunkData,
    Jpeg,
    Jpeg2000,
    H264,
    MultiZone,
    DeviceSpecific(u16),
}

impl PayloadType {
    pub fn kind(self) -> PayloadTypeKind {
        match self.0 & 0x000f {
            1 => PayloadTypeKind::Image,
            2 => PayloadTypeKind::RawData,
            3 => PayloadTypeKind::File,
            4 => PayloadTypeKind::ChunkData,
            6 => PayloadTypeKind::Jpeg,
            7 => PayloadTypeKind::Jpeg2000,
            8 => PayloadTypeKind::H264,
            9 => PayloadTypeKind::MultiZone,
            _ => PayloadTypeKind::DeviceSpecific(self.0),
        }
    }

    pub fn is_extended_chunk(self) -> bool {
        self.0.is_set(1)
    }

    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        cursor.read_bytes_be().map(Self).map_err(Into::into)
    }

    /// to be used after the generic GSVP leader packet header
    /// was parsed, before parsing the payload-specific packet
    ///
    // we need it this weird way because for some reason, the
    // potentially useful payload type specific field goes
    // BEFORE the payload type, so we have to look into the future
    // a bit, because we need to be able to see which payload type
    // we need, e. g. Image or RawData or File or etc.
    pub fn parse_generic_leader(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let position_pre = cursor.position();
        let _payload_type_specific: u16 = cursor.read_bytes_be()?;
        let res = Self::parse(cursor);
        cursor.set_position(position_pre);
        res
    }
}

pub struct ImageLeader {
    field_id: u8,
    field_count: u8,
    payload_type: PayloadType,
    timestamp: u64,
    pixel_format: PixelFormat,
    width: u32,
    height: u32,
    x_offset: u32,
    y_offset: u32,
    x_padding: u16,
    y_padding: u16,
}

impl ImageLeader {
    pub fn field_id(&self) -> u8 {
        self.field_id
    }

    pub fn field_count(&self) -> u8 {
        self.field_count
    }

    pub fn payload_type(&self) -> PayloadType {
        self.payload_type
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn x_offset(&self) -> u32 {
        self.x_offset
    }

    pub fn y_offset(&self) -> u32 {
        self.y_offset
    }

    pub fn x_padding(&self) -> u16 {
        self.x_padding
    }

    pub fn y_padding(&self) -> u16 {
        self.y_padding
    }

    pub fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let field: u8 = cursor.read_bytes_be()?;
        let field_id = field >> 4;
        let field_count = field & 0x0f;
        let _reserved: u8 = cursor.read_bytes_be()?;
        let payload_type = PayloadType::parse(cursor)?;
        let timestamp = cursor.read_bytes_be()?;
        let pixel_format = cursor
            .read_bytes_be::<u32>()?
            .try_into()
            .map_err(|e: String| Error::InvalidPacket(e.into()))?;
        let width = cursor.read_bytes_be()?;
        let height = cursor.read_bytes_be()?;
        let x_offset = cursor.read_bytes_be()?;
        let y_offset = cursor.read_bytes_be()?;
        let x_padding = cursor.read_bytes_be()?;
        let y_padding = cursor.read_bytes_be()?;
        Ok(Self {
            field_id,
            field_count,
            payload_type,
            timestamp,
            pixel_format,
            width,
            height,
            x_offset,
            y_offset,
            x_padding,
            y_padding,
        })
    }
}

pub struct ImageTrailer {}

impl ImageTrailer {
    pub fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {}
}
