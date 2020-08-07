use thiserror::Error;

use std::borrow::Cow;

pub mod channel;
pub mod device;
pub mod protocol;
pub mod register_map;

mod device_builder;

pub use device::Device;
pub use device_builder::enumerate_device;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    LibusbError(#[from] rusb::Error),

    #[error("packet is broken: {}", 0)]
    InvalidPacket(Cow<'static, str>),

    #[error("buffer io error: {}", 0)]
    BufferIoError(#[from] std::io::Error),

    #[error("device doesn't follow specification")]
    InvalidDevice,
}

pub type Result<T> = std::result::Result<T, Error>;
