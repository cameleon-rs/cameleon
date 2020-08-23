use semver::Version;
use thiserror::Error;

use crate::usb3::{register_map::*, DeviceInfo, SupportedSpeed};

use super::{
    device::Device,
    device_pool::DevicePool,
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
/// An emulator is passed to the device pool and user can't control the emulator itself directly
/// once build process is finished by calling [`build`].
///
/// Emulators in the device pool can be found by [`cameleon_device::usb3::enumerate_device`] and controlled via
/// [`cameleon_device::usb3::Device`] in the same way as real device.
///
/// [`cameleon_device::usb3::enumerate_device`]: fn.enumerate_device.html
/// [`cameleon_device::usb3::Device`]: struct.Device.html
/// [`build`]: ./struct.EmulatorBuilder.html#method.build
///
/// # Example
/// ```rust
/// use cameleon_device::usb3::{EmulatorBuilder, enumerate_device};
///
/// let mut builder = EmulatorBuilder::new();
///
/// // Build device with default configuration and pass it to the device pool.
/// // Now the device pool has one device.
/// builder.build();
///
/// // Set model name and serial number.
/// builder.model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap();
///
/// // Build device and pass it to the device pool.
/// // Now device pool has two devices.
/// builder.build();
///
/// let devices = enumerate_device().unwrap();
/// assert_eq!(devices.len(), 2);
///
/// ```
pub struct EmulatorBuilder {
    abrm: ABRM,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        Self {
            abrm: Default::default(),
        }
    }

    /// Build an emulator and pass it to the device pool. User can't control the emulator itself
    /// directly once call this method.
    ///
    /// Emulators in the device pool can be found by [`cameleon_device::usb3::enumerate_device`] and controlled via
    /// [`cameleon_device::usb3::Device`] in the same way as real device.
    ///
    /// [`cameleon_device::usb3::enumerate_device`]: fn.enumerate_device.html
    /// [`cameleon_device::usb3::Device`]: struct.Device.html
    ///
    /// # Example
    /// ```rust
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    ///
    /// // Build device with default configuration and pass it to the device pool.
    /// // Now the device pool has one device.
    /// builder.build();
    ///
    /// // Set model name and serial number.
    /// builder.model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap();
    ///
    /// // Build device and pass it to the device pool.
    /// // Now the device pool has two devices.
    /// builder.build();
    /// ```
    pub fn build(&self) {
        let memory = Memory::new(self.abrm.clone());
        let device_info = self.build_device_info();
        let device = Device::new(memory, device_info);
        DevicePool::with(|pool| pool.pool_and_run(device));
    }

    /// Setter of model name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If model name isn't set, default name is used.
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
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.model_name("my camera").is_ok());
    /// assert!(builder.model_name("私のカメラ").is_err());
    /// ```
    pub fn model_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_model_name(name)?;
        Ok(self)
    }

    /// Setter of family name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If family name isn't set, default name is used.
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
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.family_name("my camera family").is_ok());
    /// assert!(builder.family_name("私のカメラ家族").is_err());
    /// ```
    pub fn family_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_family_name(name)?;
        Ok(self)
    }

    /// Setter of serial number of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If serial number isn't set, 8 length digit is set at random.
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
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.serial_number("CAM1984").is_ok());
    /// assert!(builder.serial_number("1984年").is_err());
    /// ```
    pub fn serial_number(&mut self, serial: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_serial_number(serial)?;
        Ok(self)
    }

    /// Setter of user defined name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If user defined name isn't set, default name is set.
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
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.user_defined_name("user define name").is_ok());
    /// assert!(builder.user_defined_name("使用者定義名前").is_err());
    /// ```
    pub fn user_defined_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        self.abrm.set_user_defined_name(name)?;
        Ok(self)
    }

    fn build_device_info(&self) -> DeviceInfo {
        let gencp_version = self.abrm.version_from(abrm::GENCP_VERSION);
        let vendor_name = self.abrm.string_from(abrm::MANUFACTURER_NAME);
        let model_name = self.abrm.string_from(abrm::MODEL_NAME);
        let family_name = Some(self.abrm.string_from(abrm::FAMILY_NAME));
        let device_version = self.abrm.string_from(abrm::DEVICE_VERSION);
        let manufacturer_info = self.abrm.string_from(abrm::MANUFACTURER_INFO);
        let user_defined_name = Some(self.abrm.string_from(abrm::USER_DEFINED_NAME));
        let supported_speed = SupportedSpeed::SuperSpeed;
        let serial_number = self.abrm.string_from(abrm::SERIAL_NUMBER);

        // TODO: Read from SBRM.
        let u3v_version = Version::new(1, 0, 0);

        // Device guid consists of 12 characters.
        // First 4 characters are vendor ID and last 8 characters are unique id assigned by a vendor.
        // We use a serial number as unique id.
        let serial_len = serial_number.len();
        let guid = if serial_len > 8 {
            format!("EMU-{}", &serial_number[serial_len - 8..])
        } else {
            let pad: String = std::iter::repeat('0').take(8 - serial_len).collect();
            format!("EMU-{}{}", pad, serial_number)
        };

        DeviceInfo {
            gencp_version,
            u3v_version,
            guid,
            vendor_name,
            model_name,
            family_name,
            device_version,
            manufacturer_info,
            serial_number,
            user_defined_name,
            supported_speed,
        }
    }
}

impl Default for EmulatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
