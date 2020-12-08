pub mod u3v;

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    /// The device is busy, may be opened by another application.
    #[error("device is busy")]
    Busy,

    /// The device is disconnected from the host.
    #[error("device is disconnected")]
    Disconnected,

    /// IO error.
    #[error("input/output error: {}", 0)]
    Io(Box<dyn std::error::Error>),

    /// The device is not opened.
    #[error("device is not opened")]
    NotOpened,

    /// Device internal error.
    #[error("device internal error: {}", 0)]
    InternalError(Box<dyn std::error::Error>),

    /// Try to write invalid data to the device.
    /// e.g. try to write too large data that will overrun register.
    #[error("try to write invalid data to the device: {}", 0)]
    InvalidData(Box<dyn std::error::Error>),

    #[error("operation timed out")]
    Timeout,
}

pub type DeviceResult<T> = std::result::Result<T, DeviceError>;

#[derive(Debug, Clone, Copy)]
pub enum GenICamFileType {
    /// This is the “normal” GenICam device xml containing all device features.
    DeviceXml,
    /// This is optional XML-file that contains only the chunkdata related nodes.
    BufferXml,
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    /// Uncompressed GenICam XML file.
    Uncompressed,
    /// ZIP containing a single GenICam XML file.
    Zip,
}
