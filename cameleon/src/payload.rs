/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains types related to `Payload` sent from the device.
//!
//! `Payload` is an abstracted container that is mainly used to transfer an image, but also meta data of the image.
//! See [`Payload`] and [`ImageInfo`] for more details.

pub use cameleon_device::PixelFormat;

use std::time;

use async_channel::{Receiver, Sender};

use super::{StreamError, StreamResult};

/// Represents Payload type of the image.
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
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// A payload sent from the device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payload {
    pub(crate) id: u64,
    pub(crate) payload_type: PayloadType,
    pub(crate) image_info: Option<ImageInfo>,
    pub(crate) payload: Vec<u8>,
    pub(crate) valid_payload_size: usize,
    pub(crate) timestamp: time::Duration,
}

impl Payload {
    /// Returns [`PayloadType`] of the payload.
    pub fn payload_type(&self) -> PayloadType {
        self.payload_type
    }

    /// Returns [`ImageInfo`] if `payload_type` is [`PayloadType::Image`] or
    /// [`PayloadType::ImageExtendedChunk`].
    pub fn image_info(&self) -> Option<&ImageInfo> {
        self.image_info.as_ref()
    }

    /// Returns the image bytes in the payload if `payload_type` is [`PayloadType::Image`]  or
    /// [`PayloadType::ImageExtendedChunk`].
    pub fn image(&self) -> Option<&[u8]> {
        let image_info = self.image_info()?;
        Some(&self.payload[..image_info.image_size])
    }

    /// Returns the whole payload. Use [`Self::image`] instead if you interested only
    /// in image region of the payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.valid_payload_size]
    }

    /// Returns unique id of `payload`, which sequentially incremented every time the device send a
    /// `payload`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Timestamp of the device when the payload is generated.
    pub fn timestamp(&self) -> time::Duration {
        self.timestamp
    }

    /// Returns the payload as `Vec<u8>`.
    pub fn into_vec(mut self) -> Vec<u8> {
        self.payload.resize(self.valid_payload_size, 0);
        self.payload
    }
}

/// An Receiver of the `Payload` which is sent from a device.
#[derive(Debug, Clone)]
pub struct PayloadReceiver {
    /// Sends back `payload` to the device for reusing it.
    tx: Sender<Payload>,

    /// Receives `payload` from the device.
    rx: Receiver<StreamResult<Payload>>,
}

impl PayloadReceiver {
    /// Receives [`Payload`] sent from the device.
    pub async fn recv(&self) -> StreamResult<Payload> {
        self.rx.recv().await?
    }

    /// Tries to receive [`Payload`].
    /// This method doesn't wait arrival of `payload` and immediately returns `StreamError` if
    /// the channel is empty.
    pub fn try_recv(&self) -> StreamResult<Payload> {
        self.rx.try_recv()?
    }

    /// Receives [`Payload`] sent from the device.
    /// If the channel is empty, this method blocks until the device produces the payload.
    pub fn recv_blocking(&self) -> StreamResult<Payload> {
        self.rx.recv_blocking()?
    }

    /// Sends back [`Payload`] to the device to reuse already allocated `payload`.
    ///
    /// Sending back `payload` may improve performance of streaming, but not required to call this
    /// method.
    pub fn send_back(&self, payload: Payload) {
        self.tx.try_send(payload).ok();
    }
}

/// A sender of the [`Payload`] which is sent to the host.
#[derive(Debug, Clone)]
pub struct PayloadSender {
    /// Receives from the device.
    tx: Sender<StreamResult<Payload>>,
    /// Sends back payload to reuse it.
    rx: Receiver<Payload>,
}

impl PayloadSender {
    /// Sends [`Payload`] to the host.
    pub async fn send(&self, payload: StreamResult<Payload>) -> StreamResult<()> {
        Ok(self.tx.send(payload).await?)
    }

    /// Tries to send [`Payload`] to the host.
    /// Returns `StreamError` if the channel is full or empty.
    pub fn try_send(&self, payload: StreamResult<Payload>) -> StreamResult<()> {
        Ok(self.tx.try_send(payload)?)
    }

    /// Tries to receive [`Payload`].
    /// This method doesn't wait arrival of `payload` and immediately returns `StreamError` if
    /// the channel is empty.
    pub fn try_recv(&self) -> StreamResult<Payload> {
        Ok(self.rx.try_recv()?)
    }
}

/// Creates [`PayloadReceiver`] and [`PayloadSender`].
pub fn channel(payload_cap: usize, buffer_cap: usize) -> (PayloadSender, PayloadReceiver) {
    let (device_tx, host_rx) = async_channel::bounded(payload_cap);
    let (host_tx, device_rx) = async_channel::bounded(buffer_cap);
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

impl From<async_channel::RecvError> for StreamError {
    fn from(err: async_channel::RecvError) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl From<async_channel::TryRecvError> for StreamError {
    fn from(err: async_channel::TryRecvError) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl<T> From<async_channel::SendError<T>> for StreamError {
    fn from(err: async_channel::SendError<T>) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}

impl<T> From<async_channel::TrySendError<T>> for StreamError {
    fn from(err: async_channel::TrySendError<T>) -> Self {
        StreamError::ReceiveError(err.to_string().into())
    }
}
