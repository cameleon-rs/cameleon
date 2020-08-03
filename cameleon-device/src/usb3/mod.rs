use thiserror::Error;

pub mod control_handle;
pub mod device;

pub use device::Device;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    LibusbError(#[from] rusb::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
