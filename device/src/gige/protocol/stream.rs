use std::io;

use cameleon_impl::{bit_op::BitOp, bytes_io::ReadBytes};

use crate::gige::{Error, Result};

use super::PacketStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    status: PacketStatus,
    ei_flag: bool,
    packet_type: PacketType,
    block_id: u64,
    packet_id: u32,
    stream_flag: StreamFlag,
}

impl PacketHeader {
    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
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
