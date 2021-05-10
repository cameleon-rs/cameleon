//! This module contains types related to `Payload` sent from the device.
//!
//! `Payload` is an abstracted container that is mainly used to transfer an image, but also meta data of the image.
//! See [`Payload`] and [`ImageInfo`] for more details.

use std::time;

use async_std::channel::{Receiver, Sender};
use cameleon_device::PixelFormat;

use super::{StreamError, StreamResult};

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
    pub x_offset: usize,
    /// Y offset in pixels from the whole image origin. Some devices have capability of
    /// sending multiple extracted image regions, this fields used for the purpose.
    pub y_offset: usize,
    /// [`PixelFormat`] of the image.
    pub pixel_format: PixelFormat,
    /// Size of image in bytes.
    pub image_size: usize,
}

/// Payload sent from the device.
pub struct Payload {
    pub(crate) id: u64,
    pub(crate) payload_type: PayloadType,
    pub(crate) image_info: Option<ImageInfo>,
    pub(crate) payload: Vec<u8>,
    pub(crate) valid_payload_size: usize,
    pub(crate) timestamp: time::Duration,
}

impl Payload {
    /// Return [`PayloadType`] of the payload.
    pub fn payload_type(&self) -> PayloadType {
        self.payload_type
    }

    /// Return [`ImageInfo`] if `payload_type` is [`PayloadType::Image`] or
    /// [`PayloadType::ImageExtendedChunk`].
    pub fn image_info(&self) -> Option<&ImageInfo> {
        self.image_info.as_ref()
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
        &self.payload[..self.valid_payload_size]
    }

    /// Return unique id of `payload`, which sequentially incremented every time the device send a
    /// `payload`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Timestamp of the device when the payload is generated.
    pub fn timestamp(&self) -> time::Duration {
        self.timestamp
    }
}

/// An Receiver of the `Payload` which is sent from a device.
#[derive(Debug, Clone)]
pub struct PayloadReceiver {
    /// Send back `payload` to the device for reusing it.
    tx: Sender<Payload>,

    /// Receive `payload` from the device.
    rx: Receiver<StreamResult<Payload>>,
}

impl PayloadReceiver {
    /// Receive [`payload::Payload`] sent from the device.
    pub async fn recv(&self) -> StreamResult<Payload> {
        self.rx.recv().await?
    }

    /// Try to receive [`payload::Payload`].
    /// This method doesn't wait arrival of `payload` and immediately returns `StreamError` if
    /// the channel is empty.
    pub fn try_recv(&self) -> StreamResult<Payload> {
        self.rx.try_recv()?
    }

    /// Send back [`payload::Payload`] to the device to reuse already allocated `payload`.
    ///
    /// Sending back `payload` may improve performance of streaming, but not required to call this
    /// method.
    ///
    /// Return `StreamError` if the channel is full.
    pub fn send_back(&self, payload: Payload) -> StreamResult<()> {
        Ok(self.tx.try_send(payload)?)
    }
}

/// An Sender of the `Payload` which is sent to the host.
#[derive(Debug, Clone)]
pub struct PayloadSender {
    /// Receive from the device.
    tx: Sender<StreamResult<Payload>>,
    /// Send back payload to reuse it.
    rx: Receiver<Payload>,
}

impl PayloadSender {
    /// Send [`payload::Payload`] to the host.
    pub async fn send(&self, payload: StreamResult<Payload>) -> StreamResult<()> {
        Ok(self.tx.send(payload).await?)
    }

    /// Try to send [`payload::Payload`] to the host.
    /// Returns `StreamError` if the channel is full or empty.
    pub fn try_send(&self, payload: StreamResult<Payload>) -> StreamResult<()> {
        Ok(self.tx.try_send(payload)?)
    }

    /// Try to receive [`payload::Payload`].
    /// This method doesn't wait arrival of `payload` and immediately returns `StreamError` if
    /// the channel is empty.
    pub fn try_recv(&self) -> StreamResult<Payload> {
        Ok(self.rx.try_recv()?)
    }
}

/// Create [`PayloadReceiver`] and [`PayloadSender`].
pub fn channel(from_device_cap: usize, to_device_cap: usize) -> (PayloadSender, PayloadReceiver) {
    let (device_tx, host_rx) = async_std::channel::bounded(from_device_cap);
    let (host_tx, device_rx) = async_std::channel::bounded(to_device_cap);
    (
        PayloadSender {
            tx: device_tx,
            rx: device_rx,
        },
        PayloadReceiver {
            tx: host_tx,
            rx: host_rx,
        },
    )
}

impl From<async_std::channel::RecvError> for StreamError {
    fn from(err: async_std::channel::RecvError) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl From<async_std::channel::TryRecvError> for StreamError {
    fn from(err: async_std::channel::TryRecvError) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl<T> From<async_std::channel::SendError<T>> for StreamError {
    fn from(err: async_std::channel::SendError<T>) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl<T> From<async_std::channel::TrySendError<T>> for StreamError {
    fn from(err: async_std::channel::TrySendError<T>) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}
