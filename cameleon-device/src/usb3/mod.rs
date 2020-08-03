use thiserror::Error;

pub mod control_handle;
pub mod device;

mod device_builder;

pub use device::Device;
pub use device_builder::enumerate_device;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    LibusbError(#[from] rusb::Error),

    #[error("buffer io error: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device doesn't follow specification")]
    InvalidDevice,
}

pub type Result<T> = std::result::Result<T, Error>;
