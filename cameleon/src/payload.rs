//! This module contains tyeps related to `Payload` sent from the device.
//!
//! `Payload` is an abstracted container that is mainly used to transfer an image, but also metadata of the image.
//! See [`Payload`] and [`ImageInfo`] for more details.
use cameleon_device::PixelFormat;

/// Representing Payload type of the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadType {
    /// Payload contains just an image data only.
    Image,
    /// Payload contains multiple data chunks, and its first chunk is an image.
    ImageExtendedChunk,
    /// Payload contains multiple data chunks, no gurantee about its first chunk.
    Chunk,
}

/// Image meta information.
pub struct ImageInfo {
    /// Width of the image.
    pub width: usize,
    /// Height of the image.
    pub height: usize,
    /// X offset in pixels from the whole image origin. Some devices have capability of
    /// sending multiple extracted image regions, this fields used for the purpose.
    pub offset_x: usize,
    /// Y offset in pixels from the whole image origin. Some devices have capability of
    /// sending multiple extracted image regions, this fields used for the purpose.
    pub offset_y: usize,
    /// [`PixelFormat`] of the image.
    pub pixel_format: Option<PixelFormat>,
    /// Size of image in bytes.
    pub image_size: usize,
}

/// Payload sent from the device.
pub struct Payload {
    payload_type: PayloadType,
    info: Option<ImageInfo>,
    payload: Vec<u8>,
}

impl Payload {
    /// Return [`PayloadType`] of the payload.
    pub fn payload_type(&self) -> PayloadType {
        self.payload_type
    }

    /// Return [`ImageInfo`] if `payload_type` is [`PayloadType::Image`] or
    /// [`PayloadType::ImageExtendedChunk`].
    pub fn image_info(&self) -> Option<&ImageInfo> {
        self.info.as_ref()
    }

    /// Return the image bytes in the payload if `payload_type` is [`PayloadType::Image`]  or
    /// [`PayloadType::ImageExtendedChunk`].
    pub fn image(&self) -> Option<&[u8]> {
        let image_info = self.image_info()?;
        Some(&self.payload[..image_info.image_size])
    }

    /// Return the whole payload. Use [`Self::image`] instead if you interested only
    /// in image region of the payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}
