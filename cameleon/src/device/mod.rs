pub mod u3v;

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    /// Device is busy, may be opened by another application.
    #[error("device is busy")]
    Busy,

    /// Device is disconnected from the host.
    #[error("device is disconnected")]
    Disconnected,

    /// IO error.
    #[error("input/output error")]
    Io(Box<dyn std::error::Error>),

    /// Device is not opened.
    #[error("device is not opened")]
    NotOpened,
}

pub type DeviceResult<T> = std::result::Result<T, DeviceError>;
