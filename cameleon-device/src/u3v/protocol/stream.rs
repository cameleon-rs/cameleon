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

/// Leader of stream protocol.
///
/// # Example
/// ```no_run
/// use cameleon_device::u3v::protocol::stream::{Leader, PayloadType, ImageLeader,
///                                             ImageExtendedChunkLeader, ChunkLeader};
///
/// // Buffer for leader bytes.
/// let mut buf = Vec::new();
///
/// // Fill buffer using [`cameleon_device::u3v::Device`].
/// // ..
///
/// // Parse leader. In this point, only the generic part of the leader is parsed.
/// let leader = Leader::parse(&buf).unwrap();
///
/// // Parse a specific part of the leader.
/// match leader.payload_type() {
///     PayloadType::Image => {
///         // Try parsing specific part as Image Leader.
///         let image_leader: ImageLeader = leader.specific_leader_as().unwrap();
///     }
///     PayloadType::ImageExtendedChunk => {
///         // Try parsing specific part as Image Extended Chunk Leader.
///         let image_leader: ImageExtendedChunkLeader = leader.specific_leader_as().unwrap();
///     }
///
///     PayloadType::Chunk => {
///         // Try parsing specific part as Image Extended Chunk Leader.
///         let image_leader: ChunkLeader = leader.specific_leader_as().unwrap();
///     }
/// }
/// ```
pub struct Leader<'a> {
    leader_size: u16,
    block_id: u64,
    payload_type: PayloadType,

    /// The raw bytes represents specific leader.
    raw_specfic_leader: &'a [u8],
}

impl<'a> Leader<'a> {
    const LEADER_MAGIC: u32 = 0x4C563355;

    /// Parse bytes as Leader.
    pub fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> Result<Self> {
        let mut cursor = Cursor::new(buf.as_ref());

        Self::parse_prefix(&mut cursor)?;
        let _reserved: u16 = cursor.read_bytes()?;
        let leader_size = cursor.read_bytes()?;
        let block_id = cursor.read_bytes()?;
        let _reserved: u16 = cursor.read_bytes()?;
        let payload_type = cursor.read_bytes::<u16>()?.try_into()?;

        let raw_specfic_leader = &cursor.get_ref()[cursor.position() as usize..];

        Ok(Self {
            leader_size,
            block_id,
            payload_type,
            raw_specfic_leader,
        })
    }

    /// Return a specific part of leader.
    ///
    /// # Example
    /// ```no_run
    /// #use cameleon_device::u3v::protocol::stream::{Leader, PayloadType, ImageLeader,
    ///                                             ImageExtendedChunkLeader, ChunkLeader};
    ///
    /// #// Buffer for leader bytes.
    /// #let mut buf = Vec::new();
    ///
    /// #// Fill buffer using [`cameleon_device::u3v::Device`].
    /// #// ..
    ///
    /// #// Parse leader. In this point, only the generic part of the leader is parsed.
    /// #let leader = Leader::parse(&buf).unwrap();
    /// // Parse a specific part of the leader.
    /// match leader.payload_type() {
    ///     PayloadType::Image => {
    ///         // Try parsing specific part as Image Leader.
    ///         let image_leader: ImageLeader = leader.specific_leader_as().unwrap();
    ///     }
    ///     PayloadType::ImageExtendedChunk => {
    ///         // Try parsing specific part as Image Extended Chunk Leader.
    ///         let image_leader: ImageExtendedChunkLeader = leader.specific_leader_as().unwrap();
    ///     }
    ///
    ///     PayloadType::Chunk => {
    ///         // Try parsing specific part as Image Extended Chunk Leader.
    ///         let image_leader: ChunkLeader = leader.specific_leader_as().unwrap();
    ///     }
    /// }
    /// ```
    pub fn specific_leader_as<T: SpecificLeader>(&self) -> Result<T> {
        T::from_bytes(self.raw_specfic_leader)
    }

    /// Total size of leader, this size contains a specific leader part.
    pub fn leader_size(&self) -> u16 {
        self.leader_size
    }

    /// Type of the payload type the leader is followed by.
    pub fn payload_type(&self) -> PayloadType {
        self.payload_type
    }

    /// ID of data block.
    pub fn block_id(&self) -> u64 {
        self.block_id
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

pub trait SpecificLeader {
    fn from_bytes(buf: &[u8]) -> Result<Self>
    where
        Self: Sized;
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

/// Chunk leader is a specific leader part of stream leader.
///
/// When [`Leader::payload_type`] returns [`PayloadType::Chunk`], then the leader contains
/// [`ChunkLeader`] in a specific leader part.
pub struct ChunkLeader {
    timestamp: u64,
}

impl ChunkLeader {
    /// Timestamp when the chunk payload data is created.
    /// Timestamp represents duration since the device starts running.
    pub fn timestamp(&self) -> time::Duration {
        time::Duration::from_nanos(self.timestamp)
    }
}

impl SpecificLeader for ChunkLeader {
    fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buf);
        let timestamp = cursor.read_bytes()?;

        Ok(Self { timestamp })
    }
}

/// Trailer part of stream containing auxiliary information of payload data, which is sent after
/// the payload data.
pub struct Trailer<'a> {
    trailer_size: u16,
    block_id: u64,
    payload_status: PayloadStatus,
    valid_payload_size: u64,
    raw_specfic_trailer: &'a [u8],
}

impl<'a> Trailer<'a> {
    const TRAILER_MAGIC: u32 = 0x54563355;

    /// Parse bytes as Leader.
    pub fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> Result<Self> {
        let mut cursor = Cursor::new(buf.as_ref());

        Self::parse_prefix(&mut cursor)?;
        let _reserved: u16 = cursor.read_bytes()?;
        let trailer_size = cursor.read_bytes()?;
        let block_id = cursor.read_bytes()?;
        let payload_status = cursor.read_bytes::<u16>()?.try_into()?;
        let _reserved: u16 = cursor.read_bytes()?;
        let valid_payload_size = cursor.read_bytes()?;

        let raw_specfic_trailer = &cursor.get_ref()[cursor.position() as usize..];

        Ok(Self {
            trailer_size,
            block_id,
            payload_status,
            valid_payload_size,
            raw_specfic_trailer,
        })
    }

    /// Return a specific part of trailer.
    pub fn specific_trailer_as<T: SpecificTrailer>(&self) -> Result<T> {
        T::from_bytes(self.raw_specfic_trailer)
    }

    /// Total size of trailer, this size contains a specific trailer part.
    pub fn trailer_size(&self) -> u16 {
        self.trailer_size
    }

    /// ID of the block.
    pub fn block_id(&self) -> u64 {
        self.block_id
    }

    /// Status of payload data of the block.
    pub fn payload_status(&self) -> PayloadStatus {
        self.payload_status
    }

    /// Size of valid payload data.
    /// In case that the device send additional bytes, the additional bytes must be ignored.
    pub fn valid_payload_size(&self) -> u64 {
        self.valid_payload_size
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let magic: u32 = cursor.read_bytes()?;
        if magic == Self::TRAILER_MAGIC {
            Ok(())
        } else {
            Err(Error::InvalidPacket("invalid prefix magic".into()))
        }
    }
}

pub struct ImageTrailer {
    actual_height: u32,
}

impl ImageTrailer {
    /// Return the actual height of the payload image.
    ///
    /// Some U3V cameras support variable frame size, in that case, the height of the image may
    /// be less than or equal to the height reported in the leader.
    pub fn actual_height(&self) -> u32 {
        self.actual_height
    }
}

impl SpecificTrailer for ImageTrailer {
    fn from_bytes(mut buf: &[u8]) -> Result<Self> {
        let actual_height = buf.read_bytes()?;
        Ok(Self { actual_height })
    }
}

pub trait SpecificTrailer {
    fn from_bytes(buf: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Status of payload transfer,
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadStatus {
    /// Transfer succeed.
    Success,

    /// Some payload data is discarded.
    DataDiscarded,

    /// Some data is missed while transferring data due to inappropriate `SIRM` register settings.
    DataOverrun,
}

impl TryFrom<u16> for PayloadStatus {
    type Error = Error;

    fn try_from(val: u16) -> Result<Self> {
        match val {
            0x0000 => Ok(PayloadStatus::Success),
            0xA100 => Ok(PayloadStatus::DataDiscarded),
            0xA101 => Ok(PayloadStatus::DataOverrun),
            otherwise => Err(Error::InvalidPacket(
                format!("{} is invalid value for stream payload status", otherwise,).into(),
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

    /// Return bytes represnts generic trailer.
    fn generic_trailer_bytes(payload_type: PayloadType) -> Vec<u8> {
        let mut buf = vec![];
        let trailer_size: u16 = match payload_type {
            PayloadType::Image => 32,
            PayloadType::ImageExtendedChunk => 36,
            PayloadType::Chunk => 32,
        };

        let valid_payload_size: u64 = 4096 * 2160;
        let block_id: u64 = 51;
        // Trailer magic.
        buf.write_bytes(0x54563355u32).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Trailer size.
        buf.write_bytes(trailer_size).unwrap();
        // Block ID.
        buf.write_bytes(block_id).unwrap();
        // Status.
        buf.write_bytes(0xa100u16).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Valid paylaod size.
        buf.write_bytes(valid_payload_size).unwrap();

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
        let image_leader: ImageExtendedChunkLeader = leader.specific_leader_as().unwrap();
        assert_eq!(image_leader.timestamp(), time::Duration::from_nanos(100));
        assert_eq!(image_leader.pixel_format(), PixelFormat::BayerGR10);
        assert_eq!(image_leader.width(), 3840);
        assert_eq!(image_leader.height(), 2160);
        assert_eq!(image_leader.x_offset(), 0);
        assert_eq!(image_leader.y_offset(), 0);
        assert_eq!(image_leader.x_padding(), 0);
    }

    #[test]
    fn test_parse_chunk_leader() {
        let mut buf = generic_leader_bytes(PayloadType::Chunk);
        // Time stamp.
        buf.write_bytes(100u64).unwrap();

        let leader = Leader::parse(&buf).unwrap();
        assert_eq!(leader.payload_type(), PayloadType::Chunk);
        let image_leader: ChunkLeader = leader.specific_leader_as().unwrap();
        assert_eq!(image_leader.timestamp(), time::Duration::from_nanos(100));
    }

    #[test]
    fn test_parse_generic_trailer() {
        let mut buf = vec![];
        let trailer_size: u16 = 28;
        let block_id: u64 = 51;
        let valid_payload_size: u64 = 4096 * 2160;
        // Trailer magic.
        buf.write_bytes(0x54563355u32).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Trailer size.
        buf.write_bytes(trailer_size).unwrap();
        // Block ID.
        buf.write_bytes(block_id).unwrap();
        // Status.
        buf.write_bytes(0xa100u16).unwrap();
        // Reserved.
        buf.write_bytes(0u16).unwrap();
        // Valid paylaod size.
        buf.write_bytes(valid_payload_size).unwrap();

        let trailer = Trailer::parse(&buf).unwrap();
        assert_eq!(trailer.trailer_size(), trailer_size);
        assert_eq!(trailer.block_id(), block_id);
        assert_eq!(trailer.payload_status(), PayloadStatus::DataDiscarded);
        assert_eq!(trailer.valid_payload_size(), valid_payload_size);
    }

    #[test]
    fn test_parse_image_trailer() {
        let mut buf = generic_trailer_bytes(PayloadType::Image);

        let actual_height: u32 = 1024;
        buf.write_bytes(actual_height).unwrap();

        let trailer = Trailer::parse(&buf).unwrap();
        let image_trailer: ImageTrailer = trailer.specific_trailer_as().unwrap();
        assert_eq!(image_trailer.actual_height(), actual_height);
    }
}
