//! U3V device register classes.
//!
//! This module abstracts physical configuration of the device and provides an easy access to
//! its registers.
//!
//! # Examples
//!
//! ```no_run
//! use cameleon::device::u3v;
//! // Enumerate devices connected to the host.
//! let mut devices = u3v::enumerate_devices().unwrap();
//!
//! // If no device is connected, return.
//! if devices.is_empty() {
//!     return;
//! }
//!
//! let device = devices.pop().unwrap();
//! // Get and open handle.
//! let handle = device.control_handle();
//! handle.open().unwrap();
//!
//! // Get Abrm.
//! let abrm = handle.abrm().unwrap();
//!
//! // Read serial number from ABRM.
//! let serial_number = abrm.serial_number().unwrap();
//! println!("{}", serial_number);
//!
//! // Check user defined name feature is supported.
//! // If it is suppoted, read from and write to the register.
//! let device_capability = abrm.device_capability().unwrap();
//! if device_capability.is_user_defined_name_supported() {
//!     // Read from user defined name register.
//!     let user_defined_name = abrm.user_defined_name().unwrap().unwrap();
//!     println!("{}", user_defined_name);
//!
//!     // Write new name to the register.
//!     abrm.set_user_defined_name("cameleon").unwrap();
//! }
//! ```
use std::{convert::TryInto, time::Duration};

use cameleon_device::u3v::{self, register_map::*};

use crate::device::{CompressionType, DeviceError, DeviceResult, GenICamFileType};

use super::control_handle::ControlHandle;

/// Represent Technology Agnostic Boot Register Map (`ABRM`), refer to `GenCP` specification for more
/// information about `ABRM`.
///
/// To maintain consistency with the device data, `Abrm` doesn't cache any data. It means
/// that all methods of this struct cause communication with the device every time, thus the device
/// is expected to be opened when methods are called.
///
/// # Examples
///
/// ```no_run
/// use cameleon::device::u3v;
/// // Enumerate devices connected to the host.
/// let mut devices = u3v::enumerate_devices().unwrap();
///
/// // If no device is connected, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let device = devices.pop().unwrap();
/// // Get and open handle.
/// let handle = device.control_handle();
/// handle.open().unwrap();
///
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Read serial number from ABRM.
/// let serial_number = abrm.serial_number().unwrap();
/// println!("{}", serial_number);
///
/// // Check user defined name feature is supported.
/// // If it is suppoted, read from and write to the register.
/// let device_capability = abrm.device_capability().unwrap();
/// if device_capability.is_user_defined_name_supported() {
///     // Read from user defined name register.
///     let user_defined_name = abrm.user_defined_name().unwrap().unwrap();
///     println!("{}", user_defined_name);
///
///     // Write new name to the register.
///     abrm.set_user_defined_name("cameleon").unwrap();
/// }
///
/// ```
pub struct Abrm<'a> {
    handle: &'a ControlHandle,
    device_capability: DeviceCapability,
}

impl<'a> Abrm<'a> {
    /// Construct new `Abrm`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// use cameleon::device::u3v::register_map::Abrm;
    ///
    /// // Construct `Abrm` from control handle of the device directly.
    /// let control_handle = device.control_handle();
    /// control_handle.open().unwrap();
    /// let abrm = Abrm::new(&control_handle).unwrap();
    ///
    /// // Or `Device::abrm` can be used to construct it.
    /// let abrm = control_handle.abrm().unwrap();
    /// ```
    pub fn new(handle: &'a ControlHandle) -> DeviceResult<Self> {
        let (capability_addr, capability_len) = abrm::DEVICE_CAPABILITY;
        let device_capability = read_register(handle, capability_addr, capability_len)?;

        Ok(Self {
            handle,
            device_capability,
        })
    }

    /// Return [`Sbrm`].
    pub fn sbrm(&self) -> DeviceResult<Sbrm<'a>> {
        Sbrm::new(self.handle, self.sbrm_address()?)
    }

    /// Return [`ManifestTable`].
    pub fn manifest_table(&self) -> DeviceResult<ManifestTable<'a>> {
        Ok(ManifestTable::new(
            self.handle,
            self.manifest_table_address()?,
        ))
    }

    /// GenCP version of the device.
    pub fn gencp_version(&self) -> DeviceResult<semver::Version> {
        let gencp_version: u32 = self.read_register(abrm::GENCP_VERSION)?;
        let gencp_version_minor = gencp_version & 0xff;
        let gencp_version_major = (gencp_version >> 16) & 0xff;
        Ok(semver::Version::new(
            gencp_version_major as u64,
            gencp_version_minor as u64,
            0,
        ))
    }

    /// Manufacture name of the device.
    pub fn manufacturer_name(&self) -> DeviceResult<String> {
        self.read_register(abrm::MANUFACTURER_NAME)
    }

    /// Model name of the device.
    pub fn model_name(&self) -> DeviceResult<String> {
        self.read_register(abrm::MODEL_NAME)
    }

    /// Family name of the device.  
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    pub fn family_name(&self) -> DeviceResult<Option<String>> {
        if self.device_capability.is_family_name_supported() {
            self.read_register(abrm::FAMILY_NAME).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Device version, this information represents manufacturer specific information.
    pub fn device_version(&self) -> DeviceResult<String> {
        self.read_register(abrm::DEVICE_VERSION)
    }

    /// Manufacturer info of the device, this information represents manufacturer specific
    /// information.
    pub fn manufacturer_info(&self) -> DeviceResult<String> {
        self.read_register(abrm::MANUFACTURER_INFO)
    }

    /// Serial number of the device.
    pub fn serial_number(&self) -> DeviceResult<String> {
        self.read_register(abrm::SERIAL_NUMBER)
    }

    /// User defined name of the device.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// # let handle = device.control_handle();
    /// # handle.open().unwrap();
    /// let abrm = handle.abrm().unwrap();
    ///
    /// // Check user defined name is supported.
    /// let device_capability = abrm.device_capability().unwrap();
    /// if !device_capability.is_user_defined_name_supported() {
    ///     return;
    /// }
    ///
    /// // Read user defined name.
    /// let user_defined_name = abrm.user_defined_name().unwrap();
    ///
    /// println!("{:?}", user_defined_name);
    /// ```
    pub fn user_defined_name(&self) -> DeviceResult<Option<String>> {
        if self.device_capability.is_user_defined_name_supported() {
            self.read_register(abrm::USER_DEFINED_NAME).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Set user defined name of the device.
    ///
    /// # Arguments
    ///
    /// * `name` - A user defined name. The encoding must be ascii and the length must be less than 64.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// # let handle = device.control_handle();
    /// # handle.open().unwrap();
    /// let abrm = handle.abrm().unwrap();
    ///
    /// // Check user defined name is supported.
    /// let device_capability = abrm.device_capability().unwrap();
    /// if !device_capability.is_user_defined_name_supported() {
    ///     return;
    /// }
    ///
    /// // Write new name to the register.
    /// abrm.set_user_defined_name("cameleon").unwrap();
    /// ```
    pub fn set_user_defined_name(&self, name: &str) -> DeviceResult<()> {
        if !self.device_capability.is_user_defined_name_supported() {
            return Ok(());
        }

        self.write_register(abrm::USER_DEFINED_NAME, name)
    }

    /// The initial address of manifest table.
    ///
    /// To obtain [`ManifestTable`], it is easier to use [`Self::manifest_table`].
    pub fn manifest_table_address(&self) -> DeviceResult<u64> {
        self.read_register(abrm::MANIFEST_TABLE_ADDRESS)
    }

    /// The initial address of `Sbrm`.
    ///
    /// To obtain [`Sbrm`], it is easier to use [`Self::sbrm`].
    pub fn sbrm_address(&self) -> DeviceResult<u64> {
        self.read_register(abrm::SBRM_ADDRESS)
    }

    /// Timestamp that represents device internal clock in ns.
    ///
    /// Before calling this method, please make sure to call [`Self::set_timestamp_latch_bit`] that
    /// updates timestamp register.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// # let handle = device.control_handle();
    /// # handle.open().unwrap();
    /// let abrm = handle.abrm().unwrap();
    ///
    /// // In order to obtain current device internal clock,
    /// // make sure to call `set_timestamp_latch_bit` to
    /// // update timestamp register with current device internal clock.
    /// abrm.set_timestamp_latch_bit().unwrap();
    ///
    /// let timestamp = abrm.timestamp();
    /// ```
    pub fn timestamp(&self) -> DeviceResult<u64> {
        self.read_register(abrm::TIMESTAMP)
    }

    /// Update timestamp register by set 1 to `timestamp_latch`.
    pub fn set_timestamp_latch_bit(&self) -> DeviceResult<()> {
        self.write_register(abrm::TIMESTAMP_LATCH, 1u32)
    }

    /// Time stamp increment that indicates the ns/tick of the device internal clock.
    ///
    /// For example a value of 1000 indicates the device clock runs at 1MHz.
    pub fn timestamp_increment(&self) -> DeviceResult<u64> {
        self.read_register(abrm::TIMESTAMP_INCREMENT)
    }

    /// Device software version.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    pub fn device_software_interface_version(&self) -> DeviceResult<Option<String>> {
        if self
            .device_capability
            .is_device_software_interface_version_supported()
        {
            self.read_register(abrm::DEVICE_SOFTWARE_INTERFACE_VERSION)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    /// Maximum device response time.
    pub fn maximum_device_response_time(&self) -> DeviceResult<Duration> {
        self.read_register(abrm::MAXIMUM_DEVICE_RESPONSE_TIME)
    }

    /// Device capability.
    pub fn device_capability(&self) -> DeviceResult<DeviceCapability> {
        Ok(self.device_capability)
    }

    /// Current configuration of the device.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// # let handle = device.control_handle();
    /// # handle.open().unwrap();
    /// let abrm = handle.abrm().unwrap();
    ///
    /// let configuration = abrm.device_configuration().unwrap();
    /// if configuration.is_multi_event_enabled() {
    ///     println!("Multi event is enabled")
    /// } else {
    ///     println!("Multi event is disabled")
    /// }
    /// ```
    pub fn device_configuration(&self) -> DeviceResult<DeviceConfiguration> {
        self.read_register(abrm::DEVICE_CONFIGURATION)
    }

    /// Write configuration to the device.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cameleon::device::u3v;
    /// # let mut devices = u3v::enumerate_devices().unwrap();
    /// # let device = devices.pop().unwrap();
    /// # let handle = device.control_handle();
    /// # handle.open().unwrap();
    /// let abrm = handle.abrm().unwrap();
    ///
    /// // Check multi event feature is supported.
    /// let capability = abrm.device_capability().unwrap();
    /// if !capability.is_multi_event_supported() {
    ///     return;
    /// }
    ///
    /// // Enable multi event.
    /// let mut configuration = abrm.device_configuration().unwrap();
    /// configuration.set_multi_event_enable_bit();
    /// abrm.write_device_configuration(&configuration).unwrap();
    /// ```
    pub fn write_device_configuration(&self, config: &DeviceConfiguration) -> DeviceResult<()> {
        self.write_register(abrm::DEVICE_CONFIGURATION, config)
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        read_register(self.handle, register.0, register.1)
    }

    fn write_register(&self, register: (u64, u16), data: impl DumpBytes) -> DeviceResult<()> {
        let (addr, len) = register;
        let mut buf = vec![0; len as usize];
        data.dump_bytes(&mut buf)?;
        self.handle.write_mem(addr, &buf)
    }
}

/// Represent Technology Specific Boot Register Map (SBRM).
///
/// To maintain consistency with the device data, `Sbrm` doesn't cache any data. It means
/// that all methods of this struct cause communication with the device every time, thus the device
/// is expected to be opened when methods are called.
///
/// # Examples
///
/// ```no_run
/// use cameleon::device::u3v;
/// // Enumerate devices connected to the host.
/// let mut devices = u3v::enumerate_devices().unwrap();
///
/// // If no device is connected, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let device = devices.pop().unwrap();
/// // Get and open the handle.
/// let handle = device.control_handle();
/// handle.open().unwrap();
///
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Get Sbrm from Abrm.
/// let sbrm = abrm.sbrm().unwrap();
///
/// // Get U3V version of the device.
/// let u3v_version = sbrm.u3v_version().unwrap();
/// println!("{}", u3v_version);
/// ```
pub struct Sbrm<'a> {
    handle: &'a ControlHandle,
    sbrm_addr: u64,
    capability: U3VCapablitiy,
}

impl<'a> Sbrm<'a> {
    /// Construct new `Sbrm`.
    ///
    /// To construct `Sbrm`, it is easier to call [`Abrm::sbrm].
    pub fn new(handle: &'a ControlHandle, sbrm_addr: u64) -> DeviceResult<Self> {
        let (capability_offset, capability_len) = sbrm::U3VCP_CAPABILITY_REGISTER;
        let capability_addr = capability_offset + sbrm_addr;
        let capability = read_register(handle, capability_addr, capability_len)?;

        Ok(Self {
            handle,
            sbrm_addr,
            capability,
        })
    }

    /// Version of U3V of the device.
    pub fn u3v_version(&self) -> DeviceResult<semver::Version> {
        let u3v_version: u32 = self.read_register(sbrm::U3V_VERSION)?;
        let u3v_version_minor = u3v_version & 0xff;
        let u3v_version_major = (u3v_version >> 16) & 0xff;

        Ok(semver::Version::new(
            u3v_version_major as u64,
            u3v_version_minor as u64,
            0,
        ))
    }

    /// Maximum command transfer length in bytes.
    ///
    /// This value specifies the maximum byte length of the command which is sent from the host to
    /// the device at one time.
    pub fn maximum_command_transfer_length(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::MAXIMUM_COMMAND_TRANSFER_LENGTH)
    }

    /// Maximum acknowledge transfer length in bytes.
    ///
    /// This value specifies the maximum byte length of the acknowledge command which is sent from the device to
    /// the host at one time.
    pub fn maximum_acknowledge_trasfer_length(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH)
    }

    /// The number of stream channels the device has.
    pub fn number_of_stream_channel(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::NUMBER_OF_STREAM_CHANNELS)
    }

    /// Return [`Sirm`] if it's available.
    pub fn sirm(&self) -> DeviceResult<Option<Sirm<'a>>> {
        Ok(self
            .sirm_address()?
            .map(|addr| Sirm::new(self.handle, addr)))
    }

    /// The initial address of `Sirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn sirm_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_sirm_available() {
            self.read_register(sbrm::SIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The length of `Sirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn sirm_length(&self) -> DeviceResult<Option<u32>> {
        if self.capability.is_sirm_available() {
            self.read_register(sbrm::SIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The initial address of `Eirm`.
    ///
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn eirm_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_eirm_available() {
            self.read_register(sbrm::EIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The length of `Eirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn eirm_length(&self) -> DeviceResult<Option<u32>> {
        if self.capability.is_eirm_available() {
            self.read_register(sbrm::EIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The initial address of `IIDC2`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn iidc2_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_iidc2_available() {
            self.read_register(sbrm::IIDC2_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Current bus speed used to communication.
    pub fn current_speed(&self) -> DeviceResult<u3v::BusSpeed> {
        self.read_register(sbrm::CURRENT_SPEED)
    }

    /// Indicate some optional features are supported or not.
    pub fn u3v_capability(&self) -> DeviceResult<U3VCapablitiy> {
        Ok(self.capability)
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        let addr = offset + self.sbrm_addr;
        read_register(self.handle, addr, len)
    }
}

/// Represent Streaming Interface Register Map (SIRM).
///
/// To maintain consistency with the device data, `Sirm` doesn't cache any data. It means
/// that all methods of this struct cause communication with the device every time, thus the device
/// is expected to be opened when methods are called.
///
/// # Examples
///
/// ```no_run
/// use cameleon::device::u3v;
/// // Enumerate devices connected to the host.
/// let mut devices = u3v::enumerate_devices().unwrap();
///
/// // If no device is connected, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let device = devices.pop().unwrap();
/// // Obtain and open the handle.
/// let handle = device.control_handle();
/// handle.open().unwrap();
///
/// // Get Sirm.
/// let abrm = handle.abrm().unwrap();
/// let sbrm = abrm.sbrm().unwrap();
/// if !sbrm.u3v_capability().unwrap().is_sirm_available() {
///    return;
/// }
/// let sirm = sbrm.sirm().unwrap().unwrap();
///
/// // Enable streaming to make the device start to transmit image.
/// sirm.enable_stream().unwrap();
/// ```
pub struct Sirm<'a> {
    handle: &'a ControlHandle,
    sirm_addr: u64,
}

impl<'a> Sirm<'a> {
    /// Construct new `Sirm`.
    ///
    /// To construct `Sirm`, it's easier to call [`Sbrm::sirm`].
    pub fn new(handle: &'a ControlHandle, sirm_addr: u64) -> Self {
        Self { handle, sirm_addr }
    }

    /// Return required alignment size of payload.
    ///
    /// A host must use this value as a minimum alignment size when modifying SIRM registers
    /// related to payload size.
    pub fn payload_size_alignment(&self) -> DeviceResult<usize> {
        let si_info: u32 = self.read_register(sirm::SI_INFO)?;
        // Upper 8 bites specifies the exp of the alignment.
        let exp = si_info >> 24;
        Ok(2usize.pow(exp))
    }

    /// Enable stream.
    ///
    /// It's forbidden to write to SIRM registers while stream is enabled.
    pub fn enable_stream(&self) -> DeviceResult<()> {
        let value = 1u32;
        self.write_register(sirm::SI_CONTROL, value)
    }

    /// Disable stream.
    ///
    /// It's forbidden to write to SIRM registers while stream is enabled.
    pub fn disable_stream(&self) -> DeviceResult<()> {
        let value = 0u32;
        self.write_register(sirm::SI_CONTROL, value)
    }

    /// Return `true` if stream is enabled.
    pub fn is_stream_enable(&self) -> DeviceResult<bool> {
        let si_ctrl: u32 = self.read_register(sirm::SI_CONTROL)?;
        Ok((si_ctrl & 1) == 1)
    }

    /// Payload size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_payload_size(&self) -> DeviceResult<u64> {
        self.read_register(sirm::REQUIRED_PAYLOAD_SIZE)
    }

    /// Leader size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_leader_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::REQUIRED_LEADER_SIZE)
    }

    /// Trailer size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_trailer_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::REQUIRED_TRAILER_SIZE)
    }

    /// Maximum leader size in any device configuration.
    pub fn maximum_leader_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::MAXIMUM_LEADER_SIZE)
    }

    /// Set maximum leader size in any device configuration.
    ///
    /// A leader must be fit within one bulk transfer, so `maximum_leader_size` is restricted by the
    /// maximum size that one bulk transfer can contain.
    /// If the leader size is greater than this value in the current configuration, then device can't
    /// start streaming.
    pub fn set_maximum_leader_size(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::MAXIMUM_LEADER_SIZE, size)
    }

    /// Maximum trailer size in any device configuration.
    pub fn maximum_trailer_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::MAXIMUM_TRAILER_SIZE)
    }

    /// Set maximum trailer size in any device configuration.
    ///
    /// A trailer must be fit within one bulk transfer, so `maximum_trailer_size` is restricted by the
    /// maximum size that one bulk transfer can contain.
    /// If the trailer size is greater than this value in the current configuration, then device can't
    /// start streaming.
    pub fn set_maximum_trailer_size(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::MAXIMUM_TRAILER_SIZE, size)
    }

    /// Payload transfer size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn payload_transfer_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::PAYLOAD_TRANSFER_SIZE)
    }

    /// Set payload transfer size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn set_payload_transfer_size(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::PAYLOAD_TRANSFER_SIZE, size)
    }

    /// Payload transfer count.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn payload_transfer_count(&self) -> DeviceResult<u32> {
        self.read_register(sirm::PAYLOAD_TRANSFER_COUNT)
    }

    /// Set payload transfer count.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn set_payload_transfer_count(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::PAYLOAD_TRANSFER_COUNT, size)
    }

    /// Payload final transfer1 size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn payload_final_transfer1_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::PAYLOAD_FINAL_TRANSFER1_SIZE)
    }

    /// Set payload final transfer1 size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn set_payload_final_transfer1_size(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::PAYLOAD_FINAL_TRANSFER1_SIZE, size)
    }

    /// Payload final transfer1 size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn payload_final_transfer2_size(&self) -> DeviceResult<u32> {
        self.read_register(sirm::PAYLOAD_FINAL_TRANSFER2_SIZE)
    }

    /// Set payload final transfer1 size.
    ///
    /// Total Payload size = [`payload_transfer_size`](Self::payload_transfer_size) * [`payload_transfer_count`](Self::payload_transfer_count) + [`payload_final_transfer1_size`](Self::payload_final_transfer1_size) + [`payload_final_transfer2_size`](Self::payload_final_transfer2_size).
    pub fn set_payload_final_transfer2_size(&self, size: u32) -> DeviceResult<()> {
        self.write_register(sirm::PAYLOAD_FINAL_TRANSFER2_SIZE, size)
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        let addr = offset + self.sirm_addr;
        read_register(self.handle, addr, len)
    }

    fn write_register(&self, register: (u64, u16), data: impl DumpBytes) -> DeviceResult<()> {
        let (offset, len) = register;
        let addr = self.sirm_addr + offset;
        let mut buf = vec![0; len as usize];
        data.dump_bytes(&mut buf)?;
        self.handle.write_mem(addr, &buf)
    }
}

/// `ManifestTable` provides iterator of [`ManifestEntry`].
///
/// # Examples
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Get manifest table.
/// let manifest_table = abrm.manifest_table().unwrap();
///
/// // Iterate over entry.
/// for entry in manifest_table.entries().unwrap() {
///     // ...
/// }
/// ```
pub struct ManifestTable<'a> {
    handle: &'a ControlHandle,
    manifest_address: u64,
}

impl<'a> ManifestTable<'a> {
    /// Construct `ManifestTable` from its initial address. It may be easier to call
    /// [`Abrm::manifest_table`]
    pub fn new(handle: &'a ControlHandle, manifest_address: u64) -> Self {
        Self {
            handle,
            manifest_address,
        }
    }

    /// Return iterator of [`ManifestEntry`].
    pub fn entries(&self) -> DeviceResult<impl Iterator<Item = ManifestEntry<'a>>> {
        let entry_num: u64 = self.read_register((0, 8))?;
        let first_entry_addr = self.manifest_address + 8;
        let handle = self.handle;

        Ok((0..entry_num)
            .into_iter()
            .map(move |i| ManifestEntry::new(first_entry_addr + i * 64, handle)))
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        read_register(self.handle, offset + self.manifest_address, len)
    }
}

/// Manifest entry describes GenApi XML properties.
/// # Examples
///
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Get first manifest entry.
/// let manifest_table = abrm.manifest_table().unwrap();
/// let entry = manifest_table.entries().unwrap().next().unwrap();
///
/// // Get XML file address and length.
/// let (address, len) = (entry.file_address().unwrap(), entry.file_size().unwrap());
///
/// // Get GenICam file version.
/// let file_version = entry.genicam_file_version().unwrap();
/// ```
pub struct ManifestEntry<'a> {
    entry_addr: u64,
    handle: &'a ControlHandle,
}

impl<'a> ManifestEntry<'a> {
    /// Construct `ManifestEntry` from its initial address.
    /// Using [`ManifestTable::entries`] is recommended to obtain `ManifestEntry`.
    pub fn new(entry_addr: u64, handle: &'a ControlHandle) -> Self {
        Self { entry_addr, handle }
    }

    /// GenICam file version.
    pub fn genicam_file_version(&self) -> DeviceResult<semver::Version> {
        let file_version: u32 = self.read_register(manifest_entry::GENICAM_FILE_VERSION)?;
        let subminor = file_version & 0xff;
        let minor = (file_version >> 16) & 0xff;
        let major = (file_version >> 24) & 0xff;

        Ok(semver::Version::new(
            major as u64,
            minor as u64,
            subminor as u64,
        ))
    }

    /// Register address where GenApi XML file is located.
    pub fn file_address(&self) -> DeviceResult<u64> {
        self.read_register(manifest_entry::REGISTER_ADDRESS)
    }

    /// GenApi XML file size in bytes.
    pub fn file_size(&self) -> DeviceResult<u64> {
        self.read_register(manifest_entry::FILE_SIZE)
    }

    /// GenApi XML file info.
    pub fn file_info(&self) -> DeviceResult<GenICamFileInfo> {
        self.read_register(manifest_entry::FILE_FORMAT_INFO)
    }

    /// SHA1 hash of the file. In case the hash is not available, return None.
    pub fn sha1_hash(&self) -> DeviceResult<Option<[u8; 20]>> {
        // We don't use `self.read_register` here for perf.
        let mut sha1_hash: [u8; 20] = [0; 20];
        let addr = self.entry_addr + manifest_entry::SHA1_HASH.0;
        self.handle.read_mem(addr, &mut sha1_hash)?;

        // All bytes are 0 in case the hash is not available.
        if sha1_hash.iter().all(|byte| *byte == 0) {
            Ok(None)
        } else {
            Ok(Some(sha1_hash))
        }
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        let addr = offset + self.entry_addr;
        read_register(self.handle, addr, len)
    }
}

/// Read and parse register value.
fn read_register<T>(handle: &ControlHandle, addr: u64, len: u16) -> DeviceResult<T>
where
    T: ParseBytes,
{
    let len = len as usize;
    let mut buf = vec![0; len];
    handle.read_mem(addr, &mut buf[..len])?;
    T::parse_bytes(&buf[..len])
}

macro_rules! is_bit_set {
    ($val:expr, $bit:expr) => {
        (($val >> $bit) & 1) == 1
    };
}

macro_rules! set_bit {
    ($val:expr,  $bit:expr) => {
        $val |= (1 << $bit)
    };
}

macro_rules! unset_bit {
    ($val:expr,  $bit:expr) => {
        $val &= !(1 << $bit)
    };
}

/// Configuration of the device.
///
/// # Examples
///
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Check multi event feature is supported.
/// let capability = abrm.device_capability().unwrap();
/// if !capability.is_multi_event_supported() {
///     return;
/// }
///
/// // Enable multi event by setting configuration.
/// let mut configuration = abrm.device_configuration().unwrap();
/// configuration.set_multi_event_enable_bit();
/// abrm.write_device_configuration(&configuration).unwrap();
/// ```
#[derive(Clone, Copy)]
pub struct DeviceConfiguration(u64);
impl DeviceConfiguration {
    /// Indicate multi event is enabled on the device.
    pub fn is_multi_event_enabled(&self) -> bool {
        is_bit_set!(&self.0, 1)
    }

    /// Set multi event enable bit.
    /// To reflect the configuration change, call [`Abrm::write_device_configuration`].
    pub fn set_multi_event_enable_bit(&mut self) {
        set_bit!(self.0, 1)
    }

    /// Unset bit multi event of the device.
    /// To reflect the configuration change, call [`Abrm::write_device_configuration`].
    pub fn disable_multi_event(&mut self) {
        unset_bit!(self.0, 1)
    }
}

/// Indicate some optional features are supported or not.
///
/// # Examples
///
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Get Device Capability of the device.
/// let device_capability = abrm.device_capability().unwrap();
///
/// println!("Is user defined name supported: {}",
///     device_capability.is_user_defined_name_supported());
///
/// println!("Is family name supported: {}",
///     device_capability.is_family_name_supported());
///
/// println!("Is multi event supported: {}",
///     device_capability.is_multi_event_supported());
///
/// println!("Is device software interface version suported: {}",
///     device_capability.is_device_software_interface_version_supported());
/// ```
#[derive(Clone, Copy)]
pub struct DeviceCapability(u64);

impl DeviceCapability {
    /// Indicate whether use defined name is supported or not.
    pub fn is_user_defined_name_supported(&self) -> bool {
        is_bit_set!(&self.0, 0)
    }

    /// Indicate whether family name is supported or not.
    pub fn is_family_name_supported(&self) -> bool {
        is_bit_set!(&self.0, 8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    pub fn is_multi_event_supported(&self) -> bool {
        is_bit_set!(&self.0, 12)
    }

    /// Indicate whether the device supports stacked commands (ReadMemStacked and WriteMemStacked).
    pub fn is_stacked_commands_supported(&self) -> bool {
        is_bit_set!(&self.0, 13)
    }

    /// Indicate whether the device supports software interface version is supported.
    pub fn is_device_software_interface_version_supported(&self) -> bool {
        is_bit_set!(&self.0, 14)
    }
}

/// Indicate some optional U3V specific features are supported or not.
///
/// # Examples
///
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Sbrm.
/// let sbrm = handle.abrm().unwrap().sbrm().unwrap();
///
/// // Get U3V Capability of the device.
/// let device_capability = sbrm.u3v_capability().unwrap();
///
/// println!("Is SIRM available: {}",
///     device_capability.is_sirm_available());
///
/// println!("Is EIRM available: {}",
///     device_capability.is_eirm_available());
///
/// println!("Is iidc2 available: {}",
///     device_capability.is_iidc2_available());
///
/// ```
#[derive(Clone, Copy)]
pub struct U3VCapablitiy(u64);

impl U3VCapablitiy {
    /// Indicate whether SIRM is available or not.
    pub fn is_sirm_available(&self) -> bool {
        is_bit_set!(&self.0, 0)
    }

    /// Indicate whether EIRM is available or not.
    pub fn is_eirm_available(&self) -> bool {
        is_bit_set!(&self.0, 1)
    }

    /// Indicate whether IIDC2 is available or not.
    pub fn is_iidc2_available(&self) -> bool {
        is_bit_set!(&self.0, 2)
    }
}

/// XML file information.
///
/// # Examples
///
/// ```no_run
/// # use cameleon::device::u3v;
/// # let mut devices = u3v::enumerate_devices().unwrap();
/// # let device = devices.pop().unwrap();
/// # let handle = device.control_handle();
/// # handle.open().unwrap();
/// // Get Abrm.
/// let abrm = handle.abrm().unwrap();
///
/// // Get first manifest entry.
/// let manifest_table = abrm.manifest_table().unwrap();
/// let entry = manifest_table.entries().unwrap().next().unwrap();
///
/// // Get file info.
/// let info = entry.file_info().unwrap();
///
/// let file_type = info.file_type();
/// let compression_type = info.compression_type();
/// let schema_version = info.schema_version();
/// ```
pub struct GenICamFileInfo(u32);

impl GenICamFileInfo {
    /// Type of the XML file.
    pub fn file_type(&self) -> DeviceResult<GenICamFileType> {
        let raw = self.0 & 0b111;
        match raw {
            0 => Ok(GenICamFileType::DeviceXml),
            1 => Ok(GenICamFileType::BufferXml),
            _ => Err(DeviceError::InternalError(
                format!("Invalid U3V GenICamFileType value: {}", raw).into(),
            )),
        }
    }

    /// Compression type of the XML File.
    pub fn compression_type(&self) -> DeviceResult<CompressionType> {
        let raw = (self.0 >> 10) & 0b111111;
        match raw {
            0 => Ok(CompressionType::Uncompressed),
            1 => Ok(CompressionType::Zip),
            _ => Err(DeviceError::InternalError(
                format!("Invalid U3V GenICamFilFormat value: {}", raw).into(),
            )),
        }
    }

    /// GenICam schema version of the XML file compiles with.
    pub fn schema_version(&self) -> semver::Version {
        let major = (self.0 >> 24) & 0xff;
        let minor = (self.0 >> 16) & 0xff;
        semver::Version::new(major as u64, minor as u64, 0)
    }
}

trait ParseBytes: Sized {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self>;
}

impl ParseBytes for DeviceConfiguration {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for GenICamFileInfo {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u32::parse_bytes(bytes)?))
    }
}

impl ParseBytes for String {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        // The string may be zero-terminated.
        let len = bytes.iter().position(|&b| b == 0);
        let s = if let Some(len) = len {
            std::str::from_utf8(&bytes[..len])
        } else {
            std::str::from_utf8(bytes)
        };

        let s = s.map_err(|_| {
            DeviceError::InternalError("device's string register value is broken".into())
        })?;

        Ok(s.into())
    }
}

impl ParseBytes for Duration {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        let raw = u32::parse_bytes(bytes)?;
        Ok(Duration::from_millis(raw as u64))
    }
}

impl ParseBytes for U3VCapablitiy {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for u3v::BusSpeed {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        use u3v::BusSpeed::*;

        let raw = u32::parse_bytes(bytes)?;
        let speed = match raw {
            0b1 => LowSpeed,
            0b10 => FullSpeed,
            0b100 => HighSpeed,
            0b1000 => SuperSpeed,
            0b10000 => SuperSpeedPlus,
            other => {
                return Err(DeviceError::InternalError(
                    format!("invalid bus speed defined:  {:#b}", other).into(),
                ))
            }
        };

        Ok(speed)
    }
}

macro_rules! impl_parse_bytes_for_numeric {
    ($ty:ty) => {
        impl ParseBytes for $ty {
            fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
                let bytes = bytes.try_into().unwrap();
                Ok(<$ty>::from_le_bytes(bytes))
            }
        }
    };
}

impl_parse_bytes_for_numeric!(u8);
impl_parse_bytes_for_numeric!(u16);
impl_parse_bytes_for_numeric!(u32);
impl_parse_bytes_for_numeric!(u64);
impl_parse_bytes_for_numeric!(i8);
impl_parse_bytes_for_numeric!(i16);
impl_parse_bytes_for_numeric!(i32);
impl_parse_bytes_for_numeric!(i64);

trait DumpBytes {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()>;
}

impl<T> DumpBytes for &T
where
    T: DumpBytes,
{
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        (*self).dump_bytes(buf)
    }
}

impl DumpBytes for &str {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        if !self.is_ascii() {
            return Err(DeviceError::InvalidData(
                "string encoding must be ascii".into(),
            ));
        }

        let data_len = self.len();
        if data_len > buf.len() {
            return Err(DeviceError::InvalidData("too large string".into()));
        }

        buf[..data_len].copy_from_slice(self.as_bytes());
        // Zero terminate if data is shorter than buffer length.
        if data_len < buf.len() {
            buf[data_len] = 0;
        }

        Ok(())
    }
}

impl DumpBytes for DeviceConfiguration {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        self.0.dump_bytes(buf)
    }
}

macro_rules! impl_dump_bytes_for_numeric {
    ($ty:ty) => {
        impl DumpBytes for $ty {
            fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
                let data = self.to_le_bytes();
                debug_assert_eq!(data.len(), buf.len());

                buf.copy_from_slice(&data);
                Ok(())
            }
        }
    };
}

impl_dump_bytes_for_numeric!(u8);
impl_dump_bytes_for_numeric!(u16);
impl_dump_bytes_for_numeric!(u32);
impl_dump_bytes_for_numeric!(u64);
impl_dump_bytes_for_numeric!(i8);
impl_dump_bytes_for_numeric!(i16);
impl_dump_bytes_for_numeric!(i32);
impl_dump_bytes_for_numeric!(i64);
