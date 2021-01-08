use rand::seq::SliceRandom;
use semver::Version;
use thiserror::Error;

use crate::u3v::{BusSpeed, DeviceInfo};

use super::{
    device::Device,
    device_pool::DevicePool,
    memory::{Memory, ABRM, SBRM},
};

use cameleon_impl::memory::prelude::*;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("invalid string: {}", 0)]
    InvalidString(String),
}

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

/// USB3 emulated device builder.
/// All initial configuration of the device must be done via this builder.
///
/// An emulator is passed to the device pool and user can't control the emulator itself directly
/// once build process is finished by calling [`build`].
///
/// Emulators in the device pool can be found by [`cameleon_device::u3v::enumerate_devices`] and controlled via
/// [`cameleon_device::u3v::Device`] in the same way as real device.
///
/// [`cameleon_device::u3v::enumerate_devices`]: fn.enumerate_devices.html
/// [`cameleon_device::u3v::Device`]: struct.Device.html
/// [`build`]: ./struct.EmulatorBuilder.html#method.build
///
/// # Example
/// ```rust
/// use cameleon_device::u3v::{EmulatorBuilder, enumerate_devices};
///
/// // Build device with default configuration and pass it to the device pool.
/// // Now the device pool has one device.
/// EmulatorBuilder::new().build();
///
/// // Set model name and serial number, then build device.
/// // Now the device pool has two devices.
/// EmulatorBuilder::new().model_name("Cameleon Model").unwrap().serial_number("CAM1984").unwrap().build();
///
/// let devices = enumerate_devices().unwrap();
/// assert_eq!(devices.len(), 2);
///
/// ```
pub struct EmulatorBuilder {
    memory: Memory,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        let mut memory = Memory::new();

        // Set dummy serial number.
        let mut rang = rand::thread_rng();
        let serial_base = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let serial_number: String = (0..8)
            .map(|_| serial_base.choose(&mut rang).unwrap())
            .collect();
        memory.write::<ABRM::SerialNumber>(serial_number).unwrap();

        Self { memory }
    }

    /// Build an emulator and pass it to the device pool. User can't control the emulator itself
    /// directly once call this method.
    ///
    /// Emulators in the device pool can be found by [`cameleon_device::u3v::enumerate_devices`] and controlled via
    /// [`cameleon_device::u3v::Device`] in the same way as real device.
    ///
    /// [`cameleon_device::u3v::enumerate_devices`]: fn.enumerate_devices.html
    /// [`cameleon_device::u3v::Device`]: struct.Device.html
    ///
    /// # Example
    /// ```rust
    /// use cameleon_device::u3v::EmulatorBuilder;
    ///
    /// // Build device with default configuration and pass it to the device pool.
    /// // Now the device pool has one device.
    /// EmulatorBuilder::new().build();
    ///
    /// // Set model name and serial number, then build device.
    /// // Now the device pool has two devices.
    /// EmulatorBuilder::new().user_defined_name("My Camera").unwrap().serial_number("CAM1984").unwrap().build();
    ///
    /// ```
    pub fn build(self) {
        let device_info = self.build_device_info();
        let device = Device::new(self.memory, device_info);
        DevicePool::with(|pool| pool.pool_and_run(device));
    }

    /// Setter of serial number of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If serial number isn't set, 8 length digit is set at random.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If serial is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::u3v::EmulatorBuilder;
    ///
    /// assert!(EmulatorBuilder::new().serial_number("CAM1984").is_ok());
    /// assert!(EmulatorBuilder::new().serial_number("カム1984年").is_err());
    /// ```
    pub fn serial_number(mut self, serial: &str) -> BuilderResult<Self> {
        self.memory
            .write::<ABRM::UserDefinedName>(serial.into())
            .map_err(|e| BuilderError::InvalidString(format! {"{}", e}))?;
        Ok(self)
    }

    /// Setter of user defined name of the device. The data is flushed to ABRM segment of the device memory.
    ///
    /// If user defined name isn't set, default name is set.
    ///
    /// NOTE: Only ASCII string is accepted, and maximum string length is 64.
    ///
    /// # Errors
    /// If name is not ASCII string or the length is larger than 64, then
    /// [`BuilderError::InvalidString`] is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cameleon_device::u3v::EmulatorBuilder;
    ///
    /// assert!(EmulatorBuilder::new().user_defined_name("user define name").is_ok());
    /// assert!(EmulatorBuilder::new().user_defined_name("使用者が定義した名前").is_err());
    /// ```
    pub fn user_defined_name(mut self, name: &str) -> BuilderResult<Self> {
        self.memory
            .write::<ABRM::UserDefinedName>(name.into())
            .map_err(|e| BuilderError::InvalidString(format! {"{}", e}))?;
        Ok(self)
    }

    fn build_device_info(&self) -> DeviceInfo {
        use ABRM::*;

        let gencp_version_major = self.memory.read::<GenCpVersionMajor>().unwrap();
        let gencp_version_minor = self.memory.read::<GenCpVersionMinor>().unwrap();
        let gencp_version = Version::new(gencp_version_major as u64, gencp_version_minor as u64, 0);

        let vendor_name = self.memory.read::<ManufacturerName>().unwrap();
        let model_name = self.memory.read::<ModelName>().unwrap();
        let family_name = Some(self.memory.read::<FamilyName>().unwrap());
        let device_version = self.memory.read::<DeviceVersion>().unwrap();
        let manufacturer_info = self.memory.read::<ManufacturerInfo>().unwrap();
        let user_defined_name = Some(self.memory.read::<UserDefinedName>().unwrap());
        let supported_speed = self.current_speed();
        let serial_number = self.memory.read::<SerialNumber>().unwrap();

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

    fn current_speed(&self) -> BusSpeed {
        use BusSpeed::*;
        match self.memory.read::<SBRM::CurrentSpeed>().unwrap() {
            0b00001 => LowSpeed,
            0b00010 => FullSpeed,
            0b00100 => HighSpeed,
            0b01000 => SuperSpeed,
            0b10000 => SuperSpeedPlus,
            _ => unreachable!(),
        }
    }
}

impl Default for EmulatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
