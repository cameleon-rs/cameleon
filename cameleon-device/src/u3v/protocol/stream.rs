//! This module provides parser for U3V stream protocol.
use std::{
    convert::{TryFrom, TryInto},
    io::Cursor,
    time,
};

use crate::{
    u3v::{Error, Result},
    PixelFormat,
};

use super::util::ReadBytes;

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

    pub fn specific_leader_as<T: SpecificLeader>(&self) -> Result<T> {
        T::from_bytes(self.raw_specfic_leader)
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

pub trait SpecificLeader {
    fn from_bytes(buf: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Generic leader of stream protocol.
///
/// All specific leader follows generic leader.
struct GenericLeader {
    leader_size: u16,
    block_id: u64,
    payload_type: PayloadType,
}

impl GenericLeader {
    const LEADER_MAGIC: u32 = 0x4C563355;

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        Self::parse_prefix(cursor)?;
        let _reserved: u16 = cursor.read_bytes()?;
        let leader_size = cursor.read_bytes()?;
        let block_id = cursor.read_bytes()?;
        let _reserved: u16 = cursor.read_bytes()?;
        let payload_type = cursor.read_bytes::<u16>()?.try_into()?;

        Ok(Self {
            leader_size,
            block_id,
            payload_type,
        })
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let magic: u32 = cursor.read_bytes()?;
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

/// Image leader is a specific leader part of stream leader.
///
/// When [`Leader::payload_type`] returns [`PayloadType::Image`], then the leader contains
/// [`ImageLeader`] in a specific leader part.
pub struct ImageLeader {
    timestamp: u64,
    pixel_format: PixelFormat,
    width: u32,
    height: u32,
    x_offset: u32,
    y_offset: u32,
    x_padding: u16,
}

impl ImageLeader {
    /// Timestamp when the image is captured.
    /// Timestamp represents duration since the device starts running.
    pub fn timestamp(&self) -> time::Duration {
        time::Duration::from_nanos(self.timestamp)
    }

    /// Pixel format of the payload image.
    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    /// Width of the payload image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the payload image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// X-axis offset from the image origin.
    pub fn x_offset(&self) -> u32 {
        self.x_offset
    }

    /// Y-axis offset from the image origin.
    pub fn y_offset(&self) -> u32 {
        self.y_offset
    }

    /// Number of padding bytes added to the end of each line.
    pub fn x_padding(&self) -> u16 {
        self.x_padding
    }
}

impl SpecificLeader for ImageLeader {
    fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buf);
        let timestamp = cursor.read_bytes()?;
        let pixel_format = cursor
            .read_bytes::<u32>()?
            .try_into()
            .map_err(|e: String| Error::InvalidPacket(e.into()))?;
        let width = cursor.read_bytes()?;
        let height = cursor.read_bytes()?;
        let x_offset = cursor.read_bytes()?;
        let y_offset = cursor.read_bytes()?;
        let x_padding = cursor.read_bytes()?;
        let _reserved: u16 = cursor.read_bytes()?;

        Ok(Self {
            timestamp,
            pixel_format,
            width,
            height,
            x_offset,
            y_offset,
            x_padding,
        })
    }
}

/// Image extended chunk leader is a specific leader part of stream leader.
///
/// When [`Leader::payload_type`] returns [`PayloadType::ImageExtendedChunk`], then the leader contains
/// [`ImageExtendedChunkLeader`] in a specific leader part.
///
/// Currently, [`ImageExtendedChunkLeader`] has the same layout as [`ImageLeader`], but we
/// define this type for future changes and ergonomic design.
pub struct ImageExtendedChunkLeader {
    timestamp: u64,
    pixel_format: PixelFormat,
    width: u32,
    height: u32,
    x_offset: u32,
    y_offset: u32,
    x_padding: u16,
}

impl ImageExtendedChunkLeader {
    /// Timestamp when the image is captured.
    /// Timestamp represents duration since the device starts running.
    pub fn timestamp(&self) -> time::Duration {
        time::Duration::from_nanos(self.timestamp)
    }

    /// Pixel format of the payload image.
    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    /// Width of the payload image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the payload image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// X-axis offset from the image origin.
    pub fn x_offset(&self) -> u32 {
        self.x_offset
    }

    /// Y-axis offset from the image origin.
    pub fn y_offset(&self) -> u32 {
        self.y_offset
    }

    /// Number of padding bytes added to the end of each line.
    pub fn x_padding(&self) -> u16 {
        self.x_padding
    }
}

impl SpecificLeader for ImageExtendedChunkLeader {
    fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buf);
        let timestamp = cursor.read_bytes()?;
        let pixel_format = cursor
            .read_bytes::<u32>()?
            .try_into()
            .map_err(|e: String| Error::InvalidPacket(e.into()))?;
        let width = cursor.read_bytes()?;
        let height = cursor.read_bytes()?;
        let x_offset = cursor.read_bytes()?;
        let y_offset = cursor.read_bytes()?;
        let x_padding = cursor.read_bytes()?;
        let _reserved: u16 = cursor.read_bytes()?;

        Ok(Self {
            timestamp,
            pixel_format,
            width,
            height,
            x_offset,
            y_offset,
            x_padding,
        })
    }
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
    use super::{super::util::WriteBytes, *};

    /// Return bytes represnts generic leader.
    fn generic_leader_bytes(payload_type: PayloadType) -> Vec<u8> {
        let mut buf = vec![];
        let (payload_num, size) = match payload_type {
            PayloadType::Image => (0x0001, 50),
            PayloadType::ImageExtendedChunk => (0x4001, 50),
            PayloadType::Chunk => (0x4000, 20),
        };
        // Leader magic.
        buf.write_bytes(0x4C563355u32).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Leader size.
        buf.write_bytes(size as u16).unwrap();
        // Block_id
        buf.write_bytes(51u64).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Payload type.
        buf.write_bytes(payload_num as u16).unwrap();
        buf
    }

    #[test]
    fn test_parse_generic_leader() {
        let mut buf = vec![];
        // Leader magic.
        buf.write_bytes(0x4C563355u32).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Leader size.
        buf.write_bytes(20u16).unwrap();
        // Block ID.
        buf.write_bytes(51u64).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Payload type, Image.
        buf.write_bytes(0x0001u16).unwrap();

        let leader = Leader::parse(&buf).unwrap();
        assert_eq!(leader.leader_size(), 20);
        assert_eq!(leader.block_id(), 51);
        assert_eq!(leader.payload_type(), PayloadType::Image);
    }

    #[test]
    fn test_parse_image_leader() {
        let mut buf = generic_leader_bytes(PayloadType::Image);
        // Time stamp.
        buf.write_bytes(100u64).unwrap();
        // Pixel Format.
        buf.write_bytes::<u32>(PixelFormat::Mono8s.into()).unwrap();
        // Width.
        buf.write_bytes(3840u32).unwrap();
        // Height.
        buf.write_bytes(2160u32).unwrap();
        // X offset.
        buf.write_bytes(0u32).unwrap();
        // Y offset.
        buf.write_bytes(0u32).unwrap();
        // X padding.
        buf.write_bytes(0u16).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();

        let leader = Leader::parse(&buf).unwrap();
        assert_eq!(leader.payload_type(), PayloadType::Image);
        let image_leader: ImageLeader = leader.specific_leader_as().unwrap();
        assert_eq!(image_leader.timestamp(), time::Duration::from_nanos(100));
        assert_eq!(image_leader.pixel_format(), PixelFormat::Mono8s);
        assert_eq!(image_leader.width(), 3840);
        assert_eq!(image_leader.height(), 2160);
        assert_eq!(image_leader.x_offset(), 0);
        assert_eq!(image_leader.y_offset(), 0);
        assert_eq!(image_leader.x_padding(), 0);
    }

    #[test]
    fn test_parse_image_extended_chunk_leader() {
        let mut buf = generic_leader_bytes(PayloadType::ImageExtendedChunk);
        // Time stamp.
        buf.write_bytes(100u64).unwrap();
        // Pixel Format.
        buf.write_bytes::<u32>(PixelFormat::BayerGR10.into())
            .unwrap();
        // Width.
        buf.write_bytes(3840u32).unwrap();
        // Height.
        buf.write_bytes(2160u32).unwrap();
        // X offset.
        buf.write_bytes(0u32).unwrap();
        // Y offset.
        buf.write_bytes(0u32).unwrap();
        // X padding.
        buf.write_bytes(0u16).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();

        let leader = Leader::parse(&buf).unwrap();
        assert_eq!(leader.payload_type(), PayloadType::ImageExtendedChunk);
        let image_leader: ImageLeader = leader.specific_leader_as().unwrap();
        assert_eq!(image_leader.timestamp(), time::Duration::from_nanos(100));
        assert_eq!(image_leader.pixel_format(), PixelFormat::BayerGR10);
        assert_eq!(image_leader.width(), 3840);
        assert_eq!(image_leader.height(), 2160);
        assert_eq!(image_leader.x_offset(), 0);
        assert_eq!(image_leader.y_offset(), 0);
        assert_eq!(image_leader.x_padding(), 0);
    }
}
