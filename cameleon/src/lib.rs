#![warn(missing_docs)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc
)]

//! `cameleon` is a library for `GenICam` compatible cameras.
//! TODO: TBW

pub mod u3v;

/// The error type for manipulation of devices.
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
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
    #[error("buffer is too small to recive data")]
    BufferTooSmall,

    /// Try to write invalid data to the device.
    /// e.g. try to write too large data that will overrun register.
    #[error("try to write invalid data to the device: {0}")]
    InvalidData(Box<dyn std::error::Error>),

    /// Timeout has been occurred.
    #[error("operation timed out")]
    Timeout,
}

/// A specialized `Result` type for device manipulation.
pub type DeviceResult<T> = std::result::Result<T, DeviceError>;

/// Represent file type of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy)]
pub enum GenICamFileType {
    /// This is the “normal” `GenICam` device XML containing all device features.
    DeviceXml,
    /// This is optional XML-file that contains only the chunkdata related nodes.
    BufferXml,
}

/// Represent `CompressionType` of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    /// Uncompressed `GenICam` XML file.
    Uncompressed,
    /// ZIP containing a single `GenICam` XML file.
    Zip,
}
