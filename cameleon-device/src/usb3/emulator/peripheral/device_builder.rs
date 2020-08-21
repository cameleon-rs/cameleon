use async_std::task;
use thiserror::Error;

use super::{
    device::Device,
    device_pool::DEVICE_POOL,
    memory::{Memory, ABRM},
};

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("invalid string: {}", 0)]
    InvalidString(&'static str),
}

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

/// USB3 emulated device builder.
/// All initial configuration of the device must be done via this builder.
///
/// Built device is passed to device pool so user can't operate device itself directly
/// once build process is finished.
///
/// TODO: Add documentation of enumerate_device() and DeviceHandle.
///
/// # Example
/// ```rust
/// use cameleon_device::usb3::emulator::DeviceBuilder;
///
/// let mut builder = DeviceBuilder::new();
///
/// // Build device with default configuration and pass it to device pool.
/// // Now device pool has one device.
/// builder.build();
///
/// // Set model name and serial number.
/// builder.model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap();
///
/// // Build device and pass it to device pool.
/// // Now device pool has two devices.
/// builder.build();
/// ```
pub struct DeviceBuilder {
    abrm: ABRM,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self {
            abrm: Default::default(),
        }
    }

    /// Build device then attach it to device pool. Built device is passed to device pool and user
    /// can't operate device itself directly once this method is called.
    ///
    /// # Example
    /// ```rust
    /// use cameleon_device::usb3::emulator::DeviceBuilder;
    ///
    /// let mut builder = DeviceBuilder::new();
    ///
    /// // Build device with default configuration and pass it to device pool.
    /// // Now device pool has one device.
    /// builder.build();
    ///
    /// // Set model name and serial number.
    /// builder.model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap();
    ///
    /// // Build device and pass it to device pool.
    /// // Now device pool has two devices.
    /// builder.build();
    /// ```
    pub fn build(&self) {
        let memory = Memory::new(self.abrm.clone());
        let device = Device::new(memory);
        task::block_on(DEVICE_POOL.lock()).pool_and_run(device);
    }

    /// Setter of model name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If model name isn't set, default is used.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 63.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 63, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::emulator::DeviceBuilder;
    ///
    /// let mut builder = DeviceBuilder::new();
    /// assert!(builder.model_name("my camera").is_ok());
    /// assert!(builder.model_name("私のカメラ").is_err());
    /// ```
    pub fn model_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_model_name(name)?;
        Ok(self)
    }

    /// Setter of family name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If family name isn't set, default is used.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 63.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 63, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::emulator::DeviceBuilder;
    ///
    /// let mut builder = DeviceBuilder::new();
    /// assert!(builder.family_name("my camera family").is_ok());
    /// assert!(builder.family_name("私のカメラ家族").is_err());
    /// ```
    pub fn family_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_family_name(name)?;
        Ok(self)
    }

    /// Setter of serial number of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If serial number isn't set, default is used.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 63.
    ///
    /// # Errors
    /// If serial is not ASCII string or the length is larger than 63, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::emulator::DeviceBuilder;
    ///
    /// let mut builder = DeviceBuilder::new();
    /// assert!(builder.serial_number("CAM1984").is_ok());
    /// assert!(builder.serial_number("1984年").is_err());
    /// ```
    pub fn serial_number(&mut self, serial: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_serial_number(serial)?;
        Ok(self)
    }

    /// Setter of user defined name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If user defined name isn't set, default("") is used.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 63.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 63, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::emulator::DeviceBuilder;
    ///
    /// let mut builder = DeviceBuilder::new();
    /// assert!(builder.user_defined_name("user define name").is_ok());
    /// assert!(builder.user_defined_name("使用者定義名前").is_err());
    /// ```
    pub fn user_defined_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_user_defined_name(name)?;
        Ok(self)
    }
}

impl Default for DeviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
