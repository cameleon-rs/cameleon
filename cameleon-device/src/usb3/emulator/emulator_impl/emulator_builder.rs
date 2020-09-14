use rand::seq::SliceRandom;
use semver::Version;
use thiserror::Error;

use crate::usb3::{DeviceInfo, SupportedSpeed};

use super::{
    device::Device,
    device_pool::DevicePool,
    memory::{Memory, ABRM},
};

use cameleon_impl::memory::prelude::*;

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
    memory: Memory,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        let mut memory = Memory::new();

        // Write dummy serial number.
        let mut rang = rand::thread_rng();
        let serial_base = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let serial_number: String = (0..8)
            .map(|_| serial_base.choose(&mut rang).unwrap())
            .collect();
        memory
            .write_entry::<ABRM::SerialNumber>(serial_number)
            .unwrap();

        Self { memory }
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
    /// // Build device with default configuration and pass it to the device pool.
    /// // Now the device pool has one device.
    /// EmulatorBuilder::new().build();
    ///
    /// // Set model name and serial number.
    /// let mut builder = EmulatorBuilder::new();
    /// builder.model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap().build();
    ///
    /// // Build device and pass it to the device pool.
    /// // Now the device pool has two devices.
    /// builder.build();
    /// ```
    pub fn build(self) {
        let device_info = self.build_device_info();
        let device = Device::new(self.memory, device_info);
        DevicePool::with(|pool| pool.pool_and_run(device));
    }

    /// Setter of model name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If model name isn't set, default name is used.
    ///
    /// NOTE: Only zero-terminated ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.model_name("my camera\0").is_ok());
    /// assert!(builder.model_name("my camera").is_err());
    /// assert!(builder.model_name("私のカメラ\0").is_err());
    /// ```
    pub fn model_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        Self::assert_ascii_zero_terminated_string(name)?;
        self.memory
            .write_entry::<ABRM::ModelName>(name.into())
            .map_err(|_| BuilderError::InvalidString("string length is too large"))?;
        Ok(self)
    }

    /// Setter of family name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If family name isn't set, default name is used.
    ///
    /// NOTE: Only zero-terminated ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.family_name("my camera family\0").is_ok());
    /// assert!(builder.family_name("my camera family").is_ok());
    /// assert!(builder.family_name("私のカメラ家族\0").is_err());
    /// ```
    pub fn family_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        Self::assert_ascii_zero_terminated_string(name)?;
        self.memory
            .write_entry::<ABRM::FamilyName>(name.into())
            .map_err(|_| BuilderError::InvalidString("string length is too large"))?;
        Ok(self)
    }

    /// Setter of serial number of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If serial number isn't set, 8 length digit is set at random.
    ///
    /// NOTE: Only zero-terminated ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If serial is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.serial_number("CAM1984\0").is_ok());
    /// assert!(builder.serial_number("CAM1984").is_err());
    /// assert!(builder.serial_number("1984年\0").is_err());
    /// ```
    pub fn serial_number(&mut self, serial: &str) -> BuilderResult<&mut Self> {
        Self::assert_ascii_zero_terminated_string(serial)?;
        self.memory
            .write_entry::<ABRM::SerialNumber>(serial.into())
            .map_err(|_| BuilderError::InvalidString("string length is too large"))?;
        Ok(self)
    }

    /// Setter of user defined name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If user defined name isn't set, default name is set.
    ///
    /// NOTE: Only zero-terminated ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// [`BuilderError::InvalidString`]: enum.BuilderError.html#valirant.InvalidString
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::usb3::EmulatorBuilder;
    ///
    /// let mut builder = EmulatorBuilder::new();
    /// assert!(builder.user_defined_name("user define name\0").is_ok());
    /// assert!(builder.user_defined_name("user define name").is_err());
    /// assert!(builder.user_defined_name("使用者定義名前\0").is_err());
    /// ```
    pub fn user_defined_name(&mut self, name: &str) -> BuilderResult<&mut Self> {
        Self::assert_ascii_zero_terminated_string(name)?;
        self.memory
            .write_entry::<ABRM::UserDefinedName>(name.into())
            .map_err(|_| BuilderError::InvalidString("string length is too large"))?;
        Ok(self)
    }

    fn build_device_info(&self) -> DeviceInfo {
        use ABRM::*;
        let gencp_version_major = self.memory.read_entry::<GenCpVersionMajor>().unwrap();
        let gencp_version_minor = self.memory.read_entry::<GenCpVersionMinor>().unwrap();
        let gencp_version = Version::new(gencp_version_major as u64, gencp_version_minor as u64, 0);

        let vendor_name = self.memory.read_entry::<ManufacturerName>().unwrap();
        let model_name = self.memory.read_entry::<ModelName>().unwrap();
        let family_name = Some(self.memory.read_entry::<FamilyName>().unwrap());
        let device_version = self.memory.read_entry::<DeviceVersion>().unwrap();
        let manufacturer_info = self.memory.read_entry::<ManufacturerInfo>().unwrap();
        let user_defined_name = Some(self.memory.read_entry::<UserDefinedName>().unwrap());
        let supported_speed = SupportedSpeed::SuperSpeed;
        let serial_number = self.memory.read_entry::<SerialNumber>().unwrap();

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

    fn assert_ascii_zero_terminated_string(s: &str) -> BuilderResult<()> {
        if !s.is_empty() && s.is_ascii() && *s.as_bytes().last().unwrap() == 0 {
            Ok(())
        } else {
            Err(BuilderError::InvalidString("Not zero terminated ascii"))
        }
    }
}

impl Default for EmulatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
