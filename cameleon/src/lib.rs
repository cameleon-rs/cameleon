//! `cameleon` is a library for `GenICam` compatible cameras.
//! TODO: TBW

#![warn(missing_docs)]
#![allow(clippy::similar_names, clippy::missing_errors_doc)]

pub mod camera;
pub mod genapi;
pub mod payload;
pub mod u3v;

pub use camera::{Camera, CameraInfo, DeviceControl, PayloadStream};

use std::{borrow::Cow, num::TryFromIntError};

/// A specialized `Result` type.
pub type CameleonResult<T> = std::result::Result<T, CameleonError>;

/// An error type returned from the `camera::Camera`.
#[derive(Debug, thiserror::Error)]
pub enum CameleonError {
    /// An error from device control.
    #[error("control error: {0}")]
    ControlError(#[from] ControlError),

    /// An rrror from payload stream.
    #[error("stream error: {0}")]
    StreamError(#[from] StreamError),

    /// `GenApi` context is not laoded yet.
    #[error("`GenApi` context is missing")]
    GenApiContextMissing,

    /// `GenApi` xml doesn't meet `GenApi SFNC` specification.
    #[error("invalid `GenApi` xml: {0}")]
    InvalidGenApiXml(Cow<'static, str>),

    /// An error when `GenApi` node operation failed.
    #[error("`GenApi` error: {0}")]
    GenApiError(#[from] cameleon_genapi::GenApiError),
}

/// A specialized `Result` type for device control.
pub type ControlResult<T> = std::result::Result<T, ControlError>;

/// An error type for device control.
#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    /// The device is busy, may be opened by another application.
    #[error("device is busy")]
    Busy,

    /// The device is disconnected from the host.
    #[error("device is disconnected")]
    Disconnected,

    /// IO error.
    #[error("input/output error: {0}")]
    Io(Box<dyn std::error::Error>),

    /// The device is not opened.
    #[error("device is not opened")]
    NotOpened,

    /// Device internal error.
    #[error("device internal error: {0}")]
    InternalError(Box<dyn std::error::Error>),

    /// Buffer is too small to receive data.
    #[error("buffer is too small to recieve data")]
    BufferTooSmall,

    /// Try to write invalid data to the device, or data sent from the device is semantically invalid.
    /// e.g. try to write too large data that will overrun register.
    #[error("try to write invalid data to the device: {0}")]
    InvalidData(Box<dyn std::error::Error>),

    /// Timeout has been occurred.
    #[error("operation timed out")]
    Timeout,
}

/// A specialized `Result` type for streaming.
pub type StreamResult<T> = std::result::Result<T, StreamError>;

/// An error type related to payload streaming.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// Failed to receive [`payload::Payload`].
    #[error("failed to receive payload: {0}")]
    ReceiveError(Cow<'static, str>),

    /// Failed to send [`payload::Payload`].
    #[error("failed to send payload: {0}")]
    SendError(Cow<'static, str>),

    /// Payload leader is invalid.
    #[error("invalid payload has been sent: {0}")]
    InvalidPayload(Cow<'static, str>),

    /// Can't communicate with device.
    #[error("can't communicate the device: {0}")]
    Device(Cow<'static, str>),

    /// Buffer is too small to receive data.
    #[error("buffer is too small to recieve data")]
    BufferTooSmall,

    /// Streaming is already started.
    #[error(
        "streaming is already started. can't use the handle from the outside of streaming loop"
    )]
    InStreaming,
}

impl From<TryFromIntError> for ControlError {
    fn from(e: TryFromIntError) -> Self {
        Self::InternalError(format!("internal data has invalid num type: {}", e).into())
    }
}
