//! This module provides parser for U3V stream protocol.
use std::{
    convert::{TryFrom, TryInto},
    io::Cursor,
};

use byteorder::{ReadBytesExt, LE};

use crate::u3v::{Error, Result};

/// Leader of stream packet.
pub struct Leader<'a> {
    /// Generic leader.
    generic_leader: GenericLeader,

    /// The raw bytes represents specific leader.
    raw_specfic_leader: &'a [u8],
}

impl<'a> Leader<'a> {
    pub fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> Result<Self> {
        let mut cursor = Cursor::new(buf.as_ref());
        let generic_leader = GenericLeader::parse(&mut cursor)?;
        let raw_specfic_leader = &cursor.get_ref()[cursor.position() as usize..];
        Ok(Self {
            generic_leader,
            raw_specfic_leader,
        })
    }

    /// Total size of leader, this size contains specific leader.
    pub fn leader_size(&self) -> u16 {
        self.generic_leader.leader_size
    }

    /// Type of the payload type the leader is followed by.
    pub fn payload_type(&self) -> PayloadType {
        self.generic_leader.payload_type
    }

    /// ID of data block
    pub fn block_id(&self) -> u64 {
        self.generic_leader.block_id
    }
}

/// Generic leader of stream protocol.
///
/// All specific leader follows generic leader.
pub struct GenericLeader {
    leader_size: u16,
    block_id: u64,
    payload_type: PayloadType,
}

impl GenericLeader {
    const LEADER_MAGIC: u32 = 0x4C563355;

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        Self::parse_prefix(cursor)?;
        let _reserved = cursor.read_u16::<LE>()?;
        let leader_size = cursor.read_u16::<LE>()?;
        let block_id = cursor.read_u64::<LE>()?;
        let _reserved = cursor.read_u16::<LE>()?;
        let payload_type = cursor.read_u16::<LE>()?.try_into()?;

        Ok(Self {
            leader_size,
            block_id,
            payload_type,
        })
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let magic = cursor.read_u32::<LE>()?;
        if magic == Self::LEADER_MAGIC {
            Ok(())
        } else {
            Err(Error::InvalidPacket("invalid prefix magic".into()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    /// Type representing uncompressed image date.
    Image,

    /// Type representing uncompressed image data followed by chunk data.
    ImageExtendedChunk,

    /// Type representing chunk data.
    Chunk,
}

impl TryFrom<u16> for PayloadType {
    type Error = Error;

    fn try_from(val: u16) -> Result<Self> {
        match val {
            0x0001 => Ok(PayloadType::Image),
            0x4001 => Ok(PayloadType::ImageExtendedChunk),
            0x4000 => Ok(PayloadType::Chunk),
            val => Err(Error::InvalidPacket(
                format!("invalid value for leader payload type: {}", val).into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;

    #[test]
    fn test_parse_generic_leader() {
        let mut buf = vec![];
        // Leader magic.
        buf.write_u32::<LE>(0x4C563355).unwrap();
        // Reserved.
        buf.write_u16::<LE>(0).unwrap();
        // Leader size.
        buf.write_u16::<LE>(20).unwrap();
        // block_id
        buf.write_u64::<LE>(51).unwrap();
        // Reserved.
        buf.write_u16::<LE>(0).unwrap();
        // Payload type, Image.
        buf.write_u64::<LE>(0x0001).unwrap();

        let leader = Leader::parse(&buf).unwrap();
        assert_eq!(leader.leader_size(), 20);
        assert_eq!(leader.block_id(), 51);
        assert_eq!(leader.payload_type(), PayloadType::Image);
    }
}
