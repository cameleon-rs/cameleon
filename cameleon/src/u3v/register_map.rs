/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! U3V device register classes.
//!
//! This module abstracts physical configuration of the device and provides an easy access to
//! its registers.
//!
//! # Examples
//!
//! ```no_run
//! use cameleon::Camera;
//! use cameleon::u3v;
//! use cameleon::genapi;
//!
//! // Enumerates cameras connected to the host.
//! let mut cameras = u3v::enumerate_cameras().unwrap();
//!
//! // If no camera is found, return.
//! if cameras.is_empty() {
//!     return;
//! }
//!
//! let mut camera = cameras.pop().unwrap();
//! // Opens the camera.
//! camera.open();
//!
//! let ctrl = &mut camera.ctrl;
//! // Get Abrm.
//! let abrm = ctrl.abrm().unwrap();
//!
//! // Read serial number from ABRM.
//! let serial_number = abrm.serial_number(ctrl).unwrap();
//! println!("{}", serial_number);
//!
//! // Check user defined name feature is supported.
//! // If it is suppoted, read from and write to the register.
//! let device_capability = abrm.device_capability().unwrap();
//! if device_capability.is_user_defined_name_supported() {
//!     // Read from user defined name register.
//!     let user_defined_name = abrm.user_defined_name(ctrl).unwrap().unwrap();
//!     println!("{}", user_defined_name);
//!
//!     // Write new name to the register.
//!     abrm.set_user_defined_name(ctrl, "cameleon").unwrap();
//! }
//! ```
use std::{convert::TryInto, time::Duration};

use cameleon_device::u3v::{
    self,
    register_map::{
        abrm, manifest_entry, sbrm, sirm, DeviceCapability, DeviceConfiguration, GenICamFileInfo,
        U3VCapablitiy,
    },
};

use crate::{ControlError, ControlResult, DeviceControl};

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
/// use cameleon::Camera;
/// use cameleon::u3v;
/// use cameleon::genapi;
///
/// // Enumerates cameras connected to the host.
/// let mut cameras = u3v::enumerate_cameras().unwrap();
///
/// // If no camera is found, return.
/// if cameras.is_empty() {
///     return;
/// }
///
/// let mut camera = cameras.pop().unwrap();
/// // Opens the camera.
/// camera.open();
///
/// let ctrl = &mut camera.ctrl;
/// // Get Abrm.
/// let abrm = ctrl.abrm().unwrap();
///
/// // Read serial number from ABRM.
/// let serial_number = abrm.serial_number(ctrl).unwrap();
/// println!("{}", serial_number);
///
/// // Check user defined name feature is supported.
/// // If it is suppoted, read from and write to the register.
/// let device_capability = abrm.device_capability().unwrap();
/// if device_capability.is_user_defined_name_supported() {
///     // Read from user defined name register.
///     let user_defined_name = abrm.user_defined_name(ctrl).unwrap().unwrap();
///     println!("{}", user_defined_name);
///
///     // Write new name to the register.
///     abrm.set_user_defined_name(ctrl, "cameleon").unwrap();
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Abrm {
    device_capability: DeviceCapability,
}

impl Abrm {
    /// Constructs new `Abrm`, consider using [`super::ControlHandle::abrm`] instead.
    pub fn new<Ctrl: DeviceControl + ?Sized>(device: &mut Ctrl) -> ControlResult<Self> {
        let (capability_addr, capability_len) = abrm::DEVICE_CAPABILITY;
        let device_capability = read_register(device, capability_addr, capability_len)?;

        Ok(Self { device_capability })
    }

    /// Returns [`Sbrm`], consider using [`super::ControlHandle::sbrm`] instead.
    pub fn sbrm<Ctrl: DeviceControl + ?Sized>(&self, device: &mut Ctrl) -> ControlResult<Sbrm> {
        let sbrm_address = self.sbrm_address(device)?;
        Sbrm::new(device, sbrm_address)
    }

    /// Returns [`ManifestTable`], consider using [`super::ControlHandle::manifest_table`] instead.
    pub fn manifest_table<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<ManifestTable> {
        Ok(ManifestTable::new(self.manifest_table_address(device)?))
    }

    /// `GenCP` version of the device.
    pub fn gencp_version<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<semver::Version> {
        let gencp_version: u32 = self.read_register(device, abrm::GENCP_VERSION)?;
        let gencp_version_minor = gencp_version & 0xff;
        let gencp_version_major = (gencp_version >> 16_i32) & 0xff;
        Ok(semver::Version::new(
            u64::from(gencp_version_major),
            u64::from(gencp_version_minor),
            0,
        ))
    }

    /// Manufacture name of the device.
    pub fn manufacturer_name<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        self.read_register(device, abrm::MANUFACTURER_NAME)
    }

    /// Model name of the device.
    pub fn model_name<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        self.read_register(device, abrm::MODEL_NAME)
    }

    /// Family name of the device.  
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    pub fn family_name<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<String>> {
        if self.device_capability.is_family_name_supported() {
            self.read_register(device, abrm::FAMILY_NAME).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Device version, this information represents manufacturer specific information.
    pub fn device_version<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        self.read_register(device, abrm::DEVICE_VERSION)
    }

    /// Manufacturer info of the device, this information represents manufacturer specific
    /// information.
    pub fn manufacturer_info<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        self.read_register(device, abrm::MANUFACTURER_INFO)
    }

    /// Serial number of the device.
    pub fn serial_number<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        self.read_register(device, abrm::SERIAL_NUMBER)
    }

    /// User defined name of the device.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    pub fn user_defined_name<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<String>> {
        if self.device_capability.is_user_defined_name_supported() {
            self.read_register(device, abrm::USER_DEFINED_NAME)
                .map(Some)
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
    pub fn set_user_defined_name<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        name: &str,
    ) -> ControlResult<()> {
        if !self.device_capability.is_user_defined_name_supported() {
            return Ok(());
        }

        self.write_register(device, abrm::USER_DEFINED_NAME, name)
    }

    /// The initial address of manifest table.
    ///
    /// To obtain [`ManifestTable`], it is easier to use [`Self::manifest_table`].
    pub fn manifest_table_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u64> {
        self.read_register(device, abrm::MANIFEST_TABLE_ADDRESS)
    }

    /// The initial address of `Sbrm`.
    ///
    /// To obtain [`Sbrm`], it is easier to use [`Self::sbrm`].
    pub fn sbrm_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u64> {
        self.read_register(device, abrm::SBRM_ADDRESS)
    }

    /// Timestamp that represents device internal clock in ns.
    ///
    /// Before calling this method, please make sure to call [`Self::set_timestamp_latch_bit`] that
    /// updates timestamp register.
    pub fn timestamp<Ctrl: DeviceControl + ?Sized>(&self, device: &mut Ctrl) -> ControlResult<u64> {
        self.read_register(device, abrm::TIMESTAMP)
    }

    /// Update timestamp register by set 1 to `timestamp_latch`.
    pub fn set_timestamp_latch_bit<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<()> {
        self.write_register(device, abrm::TIMESTAMP_LATCH, 1_u32)
    }

    /// Time stamp increment that indicates the ns/tick of the device internal clock.
    ///
    /// For example a value of 1000 indicates the device clock runs at 1MHz.
    pub fn timestamp_increment<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u64> {
        self.read_register(device, abrm::TIMESTAMP_INCREMENT)
    }

    /// Device software version.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`DeviceCapability`] to see whether the feature is available on the device.
    pub fn device_software_interface_version<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<String>> {
        if self
            .device_capability
            .is_device_software_interface_version_supported()
        {
            self.read_register(device, abrm::DEVICE_SOFTWARE_INTERFACE_VERSION)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    /// Maximum device response time.
    pub fn maximum_device_response_time<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Duration> {
        self.read_register(device, abrm::MAXIMUM_DEVICE_RESPONSE_TIME)
    }

    /// Device capability.
    pub fn device_capability(&self) -> ControlResult<DeviceCapability> {
        Ok(self.device_capability)
    }

    /// Current configuration of the device.
    pub fn device_configuration<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<DeviceConfiguration> {
        self.read_register(device, abrm::DEVICE_CONFIGURATION)
    }

    /// Write configuration to the device.
    pub fn write_device_configuration<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        config: DeviceConfiguration,
    ) -> ControlResult<()> {
        self.write_register(device, abrm::DEVICE_CONFIGURATION, config)
    }

    fn read_register<T, Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        register: (u64, u16),
    ) -> ControlResult<T>
    where
        T: ParseBytes,
    {
        read_register(device, register.0, register.1)
    }

    fn write_register<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        register: (u64, u16),
        data: impl DumpBytes,
    ) -> ControlResult<()> {
        let (addr, len) = register;
        let mut buf = vec![0; len as usize];
        data.dump_bytes(&mut buf)?;
        device.write(addr, &buf)
    }
}

/// Represent Technology Specific Boot Register Map (SBRM).
///
/// To maintain consistency with the device data, `Sbrm` doesn't cache any data. It means
/// that all methods of this struct cause communication with the device every time, thus the device
/// is expected to be opened when methods are called.
#[derive(Clone, Copy, Debug)]
pub struct Sbrm {
    sbrm_addr: u64,
    capability: U3VCapablitiy,
}

impl Sbrm {
    /// Constructs new `Sbrm`, consider using [`super::ControlHandle::sbrm`] isntead.
    pub fn new<Ctrl: DeviceControl + ?Sized>(
        device: &mut Ctrl,
        sbrm_addr: u64,
    ) -> ControlResult<Self> {
        let (capability_offset, capability_len) = sbrm::U3VCP_CAPABILITY_REGISTER;
        let capability_addr = capability_offset + sbrm_addr;
        let capability = read_register(device, capability_addr, capability_len)?;

        Ok(Self {
            sbrm_addr,
            capability,
        })
    }

    /// Version of U3V of the device.
    pub fn u3v_version<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<semver::Version> {
        let u3v_version: u32 = self.read_register(device, sbrm::U3V_VERSION)?;
        let u3v_version_minor = u3v_version & 0xff;
        let u3v_version_major = (u3v_version >> 16_i32) & 0xff;

        Ok(semver::Version::new(
            u64::from(u3v_version_major),
            u64::from(u3v_version_minor),
            0,
        ))
    }

    /// Maximum command transfer length in bytes.
    ///
    /// This value specifies the maximum byte length of the command which is sent from the host to
    /// the device at one time.
    pub fn maximum_command_transfer_length<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sbrm::MAXIMUM_COMMAND_TRANSFER_LENGTH)
    }

    /// Maximum acknowledge transfer length in bytes.
    ///
    /// This value specifies the maximum byte length of the acknowledge command which is sent from the device to
    /// the host at one time.
    pub fn maximum_acknowledge_trasfer_length<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sbrm::MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH)
    }

    /// The number of stream channels the device has.
    pub fn number_of_stream_channel<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sbrm::NUMBER_OF_STREAM_CHANNELS)
    }

    /// Return [`Sirm`] if it's available.
    pub fn sirm<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<Sirm>> {
        Ok(self.sirm_address(device)?.map(Sirm::new))
    }

    /// The initial address of `Sirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn sirm_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<u64>> {
        if self.capability.is_sirm_available() {
            self.read_register(device, sbrm::SIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The length of `Sirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn sirm_length<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<u32>> {
        if self.capability.is_sirm_available() {
            self.read_register(device, sbrm::SIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The initial address of `Eirm`.
    ///
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn eirm_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<u64>> {
        if self.capability.is_eirm_available() {
            self.read_register(device, sbrm::EIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The length of `Eirm`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn eirm_length<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<u32>> {
        if self.capability.is_eirm_available() {
            self.read_register(device, sbrm::EIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The initial address of `IIDC2`.
    ///
    /// NOTE: Some device doesn't support this feature.
    /// Please refer to [`U3VCapablitiy`] to see whether the feature is available on the device.
    pub fn iidc2_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<u64>> {
        if self.capability.is_iidc2_available() {
            self.read_register(device, sbrm::IIDC2_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Current bus speed used to communication.
    pub fn current_speed<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u3v::BusSpeed> {
        self.read_register(device, sbrm::CURRENT_SPEED)
    }

    /// Indicate some optional features are supported or not.
    pub fn u3v_capability(&self) -> ControlResult<U3VCapablitiy> {
        Ok(self.capability)
    }

    fn read_register<T, Ctrl>(&self, device: &mut Ctrl, register: (u64, u16)) -> ControlResult<T>
    where
        T: ParseBytes,
        Ctrl: DeviceControl + ?Sized,
    {
        let (offset, len) = register;
        let addr = offset + self.sbrm_addr;
        read_register(device, addr, len)
    }
}

/// Represent Streaming Interface Register Map (SIRM).
///
/// To maintain consistency with the device data, `Sirm` doesn't cache any data. It means
/// that all methods of this struct cause communication with the device every time, thus the device
/// is expected to be opened when methods are called.
#[derive(Clone, Copy, Debug)]
pub struct Sirm {
    sirm_addr: u64,
}

impl Sirm {
    /// Constructs new `Sirm`, consider using [`super::ControlHandle::sirm`] instead.
    ///
    /// To construct `Sirm`, Use [`Sbrm::sirm`] also can be used.
    #[must_use]
    pub fn new(sirm_addr: u64) -> Self {
        Self { sirm_addr }
    }

    /// Returns required alignment size of payload.
    ///
    /// A host must use this value as a minimum alignment size when modifying SIRM registers
    /// related to payload size.
    pub fn payload_size_alignment<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<usize> {
        let si_info: u32 = self.read_register(device, sirm::SI_INFO)?;
        // Upper 8 bits specifies the exp of the alignment.
        Ok(1 << (si_info >> 24_i32))
    }

    /// Enables stream.
    ///
    /// It's forbidden to write to SIRM registers while stream is enabled.
    pub fn enable_stream<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<()> {
        let value = 1_u32;
        self.write_register(device, sirm::SI_CONTROL, value)
    }

    /// Disables stream.
    ///
    /// It's forbidden to write to SIRM registers while stream is enabled.
    pub fn disable_stream<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<()> {
        let value = 0_u32;
        self.write_register(device, sirm::SI_CONTROL, value)
    }

    /// Returns `true` if stream is enabled.
    pub fn is_stream_enable<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<bool> {
        let si_ctrl: u32 = self.read_register(device, sirm::SI_CONTROL)?;
        Ok((si_ctrl & 1) == 1)
    }

    /// Payload size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_payload_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u64> {
        self.read_register(device, sirm::REQUIRED_PAYLOAD_SIZE)
    }

    /// Leader size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_leader_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::REQUIRED_LEADER_SIZE)
    }

    /// Trailer size of an image or chunk data in current device configuration.
    ///
    /// This value is never changed while stream is enabled.
    /// Once stream is disabled, the value may be changed, so The host must reload the value to
    /// update the buffer size required for payload data.
    pub fn required_trailer_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::REQUIRED_TRAILER_SIZE)
    }

    /// Maximum leader size in any device configuration.
    pub fn maximum_leader_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::MAXIMUM_LEADER_SIZE)
    }

    /// Sets maximum leader size in any device configuration.
    ///
    /// A leader must be fit within one bulk transfer, so `maximum_leader_size` is restricted by the
    /// maximum size that one bulk transfer can contain.
    /// If the leader size is greater than this value in the current configuration, then device can't
    /// start streaming.
    pub fn set_maximum_leader_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::MAXIMUM_LEADER_SIZE, size)
    }

    /// Maximum trailer size in any device configuration.
    pub fn maximum_trailer_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::MAXIMUM_TRAILER_SIZE)
    }

    /// Sets maximum trailer size in any device configuration.
    ///
    /// A trailer must be fit within one bulk transfer, so `maximum_trailer_size` is restricted by the
    /// maximum size that one bulk transfer can contain.
    /// If the trailer size is greater than this value in the current configuration, then device can't
    /// start streaming.
    pub fn set_maximum_trailer_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::MAXIMUM_TRAILER_SIZE, size)
    }

    /// Payload transfer size.
    pub fn payload_transfer_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::PAYLOAD_TRANSFER_SIZE)
    }

    /// Set payload transfer size.
    pub fn set_payload_transfer_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::PAYLOAD_TRANSFER_SIZE, size)
    }

    /// Payload transfer count.
    pub fn payload_transfer_count<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::PAYLOAD_TRANSFER_COUNT)
    }

    /// Sets payload transfer count.
    pub fn set_payload_transfer_count<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::PAYLOAD_TRANSFER_COUNT, size)
    }

    /// Payload final transfer1 size.
    pub fn payload_final_transfer1_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::PAYLOAD_FINAL_TRANSFER1_SIZE)
    }

    /// Sets payload final transfer1 size.
    pub fn set_payload_final_transfer1_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::PAYLOAD_FINAL_TRANSFER1_SIZE, size)
    }

    /// Payload final transfer1 size.
    pub fn payload_final_transfer2_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        self.read_register(device, sirm::PAYLOAD_FINAL_TRANSFER2_SIZE)
    }

    /// Set payload final transfer1 size.
    pub fn set_payload_final_transfer2_size<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        size: u32,
    ) -> ControlResult<()> {
        self.write_register(device, sirm::PAYLOAD_FINAL_TRANSFER2_SIZE, size)
    }

    fn read_register<T, Ctrl>(&self, device: &mut Ctrl, register: (u64, u16)) -> ControlResult<T>
    where
        T: ParseBytes,
        Ctrl: DeviceControl + ?Sized,
    {
        let (offset, len) = register;
        let addr = offset + self.sirm_addr;
        read_register(device, addr, len)
    }

    fn write_register<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        register: (u64, u16),
        data: impl DumpBytes,
    ) -> ControlResult<()> {
        let (offset, len) = register;
        let addr = self.sirm_addr + offset;
        let mut buf = vec![0; len as usize];
        data.dump_bytes(&mut buf)?;
        device.write(addr, &buf)
    }
}

/// `ManifestTable` provides iterator of [`ManifestEntry`].
#[derive(Clone, Copy, Debug)]
pub struct ManifestTable {
    manifest_address: u64,
}

impl ManifestTable {
    /// Constructs new `ManifestEntry`, consider using [`super::ControlHandle::manifest_table`]
    /// instead.
    #[must_use]
    pub fn new(manifest_address: u64) -> Self {
        Self { manifest_address }
    }

    /// Returns iterator of [`ManifestEntry`].
    pub fn entries<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<impl Iterator<Item = ManifestEntry>> {
        let entry_num: u64 = self.read_register(device, (0, 8))?;
        let first_entry_addr = self.manifest_address + 8;

        Ok((0..entry_num)
            .into_iter()
            .map(move |i| ManifestEntry::new(first_entry_addr + i * 64)))
    }

    fn read_register<T, Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
        register: (u64, u16),
    ) -> ControlResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        read_register(device, offset + self.manifest_address, len)
    }
}

/// Manifest entry describes `GenApi` XML properties.
#[derive(Clone, Copy, Debug)]
pub struct ManifestEntry {
    entry_addr: u64,
}

impl ManifestEntry {
    /// Construct `ManifestEntry` from its initial address.
    /// Using [`ManifestTable::entries`] is recommended to obtain `ManifestEntry`.
    #[must_use]
    pub fn new(entry_addr: u64) -> Self {
        Self { entry_addr }
    }

    /// `GenICam` file version.
    pub fn genicam_file_version<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<semver::Version> {
        let file_version: u32 = self.read_register(device, manifest_entry::GENICAM_FILE_VERSION)?;
        let subminor = file_version & 0xff;
        let minor = (file_version >> 16_i32) & 0xff;
        let major = (file_version >> 24_i32) & 0xff;

        Ok(semver::Version::new(
            u64::from(major),
            u64::from(minor),
            u64::from(subminor),
        ))
    }

    /// Register address where `GenApi` XML file is located.
    pub fn file_address<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<u64> {
        self.read_register(device, manifest_entry::REGISTER_ADDRESS)
    }

    /// `GenApi` XML file size in bytes.
    pub fn file_size<Ctrl: DeviceControl + ?Sized>(&self, device: &mut Ctrl) -> ControlResult<u64> {
        self.read_register(device, manifest_entry::FILE_SIZE)
    }

    /// `GenApi` XML file info.
    pub fn file_info<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<GenICamFileInfo> {
        self.read_register(device, manifest_entry::FILE_FORMAT_INFO)
    }

    /// SHA1 hash of the file. In case the hash is not available, return None.
    pub fn sha1_hash<Ctrl: DeviceControl + ?Sized>(
        &self,
        device: &mut Ctrl,
    ) -> ControlResult<Option<[u8; 20]>> {
        // We don't use `self.read_register` here for perf.
        let mut sha1_hash: [u8; 20] = [0; 20];
        let addr = self.entry_addr + manifest_entry::SHA1_HASH.0;
        device.read(addr, &mut sha1_hash)?;

        // All bytes are 0 in case the hash is not available.
        if sha1_hash.iter().all(|byte| *byte == 0) {
            Ok(None)
        } else {
            Ok(Some(sha1_hash))
        }
    }

    fn read_register<T, Ctrl>(&self, device: &mut Ctrl, register: (u64, u16)) -> ControlResult<T>
    where
        T: ParseBytes,
        Ctrl: DeviceControl + ?Sized,
    {
        let (offset, len) = register;
        let addr = offset + self.entry_addr;
        read_register(device, addr, len)
    }
}

/// Reads and parses register value.
fn read_register<T, Ctrl: DeviceControl + ?Sized>(
    device: &mut Ctrl,
    addr: u64,
    len: u16,
) -> ControlResult<T>
where
    T: ParseBytes,
{
    let len = len as usize;
    let mut buf = vec![0; len];
    device.read(addr, &mut buf[..len])?;
    T::parse_bytes(&buf[..len])
}

/// Represent file type of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenICamFileType {
    /// This is the “normal” `GenICam` device XML containing all device features.
    DeviceXml,
    /// This is optional XML-file that contains only the chunkdata related nodes.
    BufferXml,
}

trait ParseBytes: Sized {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self>;
}

impl ParseBytes for DeviceConfiguration {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for GenICamFileInfo {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        Ok(Self(u32::parse_bytes(bytes)?))
    }
}

impl ParseBytes for String {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        // The string may be zero-terminated.
        let len = bytes.iter().position(|&b| b == 0);
        let s = len.map_or_else(
            || std::str::from_utf8(bytes),
            |len| std::str::from_utf8(&bytes[..len]),
        );

        let s = s.map_err(|_| {
            ControlError::InvalidDevice("device's string register value is broken".into())
        })?;

        Ok(s.into())
    }
}

impl ParseBytes for Duration {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        let raw = u32::parse_bytes(bytes)?;
        Ok(Duration::from_millis(u64::from(raw)))
    }
}

impl ParseBytes for U3VCapablitiy {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for u3v::BusSpeed {
    fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
        use u3v::BusSpeed::{FullSpeed, HighSpeed, LowSpeed, SuperSpeed, SuperSpeedPlus};

        let raw = u32::parse_bytes(bytes)?;
        let speed = match raw {
            0b1 => LowSpeed,
            0b10 => FullSpeed,
            0b100 => HighSpeed,
            0b1000 => SuperSpeed,
            0b10000 => SuperSpeedPlus,
            other => {
                return Err(ControlError::InvalidDevice(
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
            fn parse_bytes(bytes: &[u8]) -> ControlResult<Self> {
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
    fn dump_bytes(&self, buf: &mut [u8]) -> ControlResult<()>;
}

impl<T> DumpBytes for &T
where
    T: DumpBytes,
{
    fn dump_bytes(&self, buf: &mut [u8]) -> ControlResult<()> {
        (*self).dump_bytes(buf)
    }
}

impl DumpBytes for &str {
    fn dump_bytes(&self, buf: &mut [u8]) -> ControlResult<()> {
        if !self.is_ascii() {
            return Err(ControlError::InvalidData(
                "string encoding must be ascii".into(),
            ));
        }

        let data_len = self.len();
        if data_len > buf.len() {
            return Err(ControlError::InvalidData("too large string".into()));
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
    fn dump_bytes(&self, buf: &mut [u8]) -> ControlResult<()> {
        self.0.dump_bytes(buf)
    }
}

macro_rules! impl_dump_bytes_for_numeric {
    ($ty:ty) => {
        impl DumpBytes for $ty {
            fn dump_bytes(&self, buf: &mut [u8]) -> ControlResult<()> {
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
