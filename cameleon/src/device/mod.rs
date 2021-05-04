//! This module provides low level API for manipulating `GenICam` compatible devices.
//!
//! # Examples
//!
//! ```no_run
//! use cameleon::device::u3v;
//! // Enumerate devices connected to the host.
//! let mut devices = u3v::enumerate_devices().unwrap();
//!
//! // If no device is connected, return.
//! if devices.is_empty() {
//!     return;
//! }
//!
//! let mut device = devices.pop().unwrap();
//! // Get control handle of the device.
//! let control_handle = device.control_handle();
//! control_handle.open().unwrap();
//!
//! // Get Abrm.
//! let abrm = control_handle.abrm().unwrap();
//!
//! // Read serial number from ABRM.
//! let serial_number = abrm.serial_number().unwrap();
//! println!("{}", serial_number);
//!
//! // Check user defined name feature is supported.
//! // If it is suppoted, read from and write to the register.
//! let device_capability = abrm.device_capability().unwrap();
//! if device_capability.is_user_defined_name_supported() {
//!     // Read from user defined name register.
//!     let user_defined_name = abrm.user_defined_name().unwrap().unwrap();
//!     println!("{}", user_defined_name);
//!
//!     // Write new name to the register.
//!     abrm.set_user_defined_name("cameleon").unwrap();
//! }
//! ```
pub mod u3v;

/// The error type for manipulation of devices.
///
/// Errors mostly originate from connection layer between the host and the device, e.g. USB layer.
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
