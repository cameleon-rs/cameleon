pub mod device;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error which occurs in low level layer.
    #[error("device error: {}", 0)]
    DeviceError(#[from] device::DeviceError),
}

pub type Result<T> = std::result::Result<T, Error>;
