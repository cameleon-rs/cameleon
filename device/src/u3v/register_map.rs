/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::bit_op::BitOp;

use super::{Error, Result};
use crate::{CompressionType, GenICamFileType};

/// (Address, Length) of registers in Technology Agnostic Boot Register Map (ABRM).
pub mod abrm {
    pub const GENCP_VERSION: (u64, u16) = (0x0000, 4);
    pub const MANUFACTURER_NAME: (u64, u16) = (0x0004, 64);
    pub const MODEL_NAME: (u64, u16) = (0x0044, 64);
    pub const FAMILY_NAME: (u64, u16) = (0x0084, 64);
    pub const DEVICE_VERSION: (u64, u16) = (0x00C4, 64);
    pub const MANUFACTURER_INFO: (u64, u16) = (0x0104, 64);
    pub const SERIAL_NUMBER: (u64, u16) = (0x0144, 64);
    pub const USER_DEFINED_NAME: (u64, u16) = (0x0184, 64);
    pub const DEVICE_CAPABILITY: (u64, u16) = (0x01C4, 8);
    pub const MAXIMUM_DEVICE_RESPONSE_TIME: (u64, u16) = (0x01CC, 4);
    pub const MANIFEST_TABLE_ADDRESS: (u64, u16) = (0x01D0, 8);
    pub const SBRM_ADDRESS: (u64, u16) = (0x01D8, 8);
    pub const DEVICE_CONFIGURATION: (u64, u16) = (0x01E0, 8);
    pub const HEARTBEAT_TIMEOUT: (u64, u16) = (0x01E8, 4);
    pub const MESSAGE_CHANNEL_ID: (u64, u16) = (0x01EC, 4);
    pub const TIMESTAMP: (u64, u16) = (0x01F0, 8);
    pub const TIMESTAMP_LATCH: (u64, u16) = (0x01F8, 4);
    pub const TIMESTAMP_INCREMENT: (u64, u16) = (0x01FC, 8);
    pub const ACCESS_PRIVILEGE: (u64, u16) = (0x0204, 4);
    pub const PROTOCOL_ENDIANNESS: (u64, u16) = (0x0208, 4);
    pub const IMPLEMENTATION_ENDIANNESS: (u64, u16) = (0x020C, 4);
    pub const DEVICE_SOFTWARE_INTERFACE_VERSION: (u64, u16) = (0x0210, 64);
}

/// (Offset, Length) of registers in Technology Specific Boot Register Map (SBRM).
/// SBRM base address can be obtained by reading `abrm::SBRM_ADDRESS`.
pub mod sbrm {
    pub const U3V_VERSION: (u64, u16) = (0x0000, 4);
    pub const U3VCP_CAPABILITY_REGISTER: (u64, u16) = (0x0004, 8);
    pub const U3VCP_CONFIGURATION_REGISTER: (u64, u16) = (0x000C, 8);
    pub const MAXIMUM_COMMAND_TRANSFER_LENGTH: (u64, u16) = (0x014, 4);
    pub const MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH: (u64, u16) = (0x018, 4);
    pub const NUMBER_OF_STREAM_CHANNELS: (u64, u16) = (0x01C, 4);
    pub const SIRM_ADDRESS: (u64, u16) = (0x020, 8);
    pub const SIRM_LENGTH: (u64, u16) = (0x028, 4);
    pub const EIRM_ADDRESS: (u64, u16) = (0x02C, 8);
    pub const EIRM_LENGTH: (u64, u16) = (0x034, 4);
    pub const IIDC2_ADDRESS: (u64, u16) = (0x038, 8);
    pub const CURRENT_SPEED: (u64, u16) = (0x040, 4);
}

/// (Offset, Length) of registers in Event Interface Register Map (EIRM).
/// SIRM base address can be obtained by
/// [`sbrm::EIRM_ADDRESS`].
pub mod eirm {
    pub const EI_CONTROL: (u64, u16) = (0x0000, 4);
    pub const MAXIMUM_EVENT_TRANSFER_LENGTH: (u64, u16) = (0x0004, 4);
    pub const EVENT_TEST_CONTROL: (u64, u16) = (0x0008, 4);
}

/// (Offset, Length) of registers in Streaming Interface Register Map (SIRM).
/// SIRM base address can be obtained by
/// `sbrm::SIRM_ADDRESS`.
pub mod sirm {
    pub const SI_INFO: (u64, u16) = (0x0000, 4);
    pub const SI_CONTROL: (u64, u16) = (0x0004, 4);
    pub const REQUIRED_PAYLOAD_SIZE: (u64, u16) = (0x0008, 8);
    pub const REQUIRED_LEADER_SIZE: (u64, u16) = (0x0010, 4);
    pub const REQUIRED_TRAILER_SIZE: (u64, u16) = (0x0014, 4);
    pub const MAXIMUM_LEADER_SIZE: (u64, u16) = (0x0018, 4);
    pub const PAYLOAD_TRANSFER_SIZE: (u64, u16) = (0x001C, 4);
    pub const PAYLOAD_TRANSFER_COUNT: (u64, u16) = (0x0020, 4);
    pub const PAYLOAD_FINAL_TRANSFER1_SIZE: (u64, u16) = (0x0024, 4);
    pub const PAYLOAD_FINAL_TRANSFER2_SIZE: (u64, u16) = (0x0028, 4);
    pub const MAXIMUM_TRAILER_SIZE: (u64, u16) = (0x002C, 4);
}

/// (Offset, Length) of registers in a manifest entry.
pub mod manifest_entry {
    pub const GENICAM_FILE_VERSION: (u64, u16) = (0x0000, 4);
    /// Information about schema version, file type and file format is serialized into 32 bit field.
    pub const FILE_FORMAT_INFO: (u64, u16) = (0x0004, 4);
    pub const REGISTER_ADDRESS: (u64, u16) = (0x0008, 8);
    pub const FILE_SIZE: (u64, u16) = (0x0010, 8);
    pub const SHA1_HASH: (u64, u16) = (0x0018, 20);
}

/// Configuration of the device.
#[derive(Clone, Copy, Debug)]
pub struct DeviceConfiguration(pub u64);
impl DeviceConfiguration {
    /// Indicate multi event is enabled on the device.
    #[must_use]
    pub fn is_multi_event_enabled(self) -> bool {
        self.0.is_set(1)
    }

    /// Sets multi event enable bit.
    pub fn set_multi_event_enable_bit(&mut self) {
        self.0 = self.0.set_bit(1)
    }

    /// Unsets bit multi event of the device.
    pub fn disable_multi_event(&mut self) {
        self.0 = self.0.clear_bit(1)
    }
}

/// Indicate some optional features are supported or not.
#[derive(Clone, Copy, Debug)]
pub struct DeviceCapability(pub u64);

impl DeviceCapability {
    /// Indicate whether use defined name is supported or not.
    #[must_use]
    pub fn is_user_defined_name_supported(self) -> bool {
        self.0.is_set(0)
    }

    /// Indicate whether family name is supported or not.
    #[must_use]
    pub fn is_family_name_supported(self) -> bool {
        self.0.is_set(8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    #[must_use]
    pub fn is_multi_event_supported(self) -> bool {
        self.0.is_set(12)
    }

    /// Indicate whether the device supports stacked commands (`ReadMemStacked` and `WriteMemStacked`).
    #[must_use]
    pub fn is_stacked_commands_supported(self) -> bool {
        self.0.is_set(13)
    }

    /// Indicate whether the device supports software interface version is supported.
    #[must_use]
    pub fn is_device_software_interface_version_supported(self) -> bool {
        self.0.is_set(14)
    }
}

/// Indicate some optional U3V specific features are supported or not.
#[derive(Clone, Copy, Debug)]
pub struct U3VCapablitiy(pub u64);

impl U3VCapablitiy {
    /// Indicate whether SIRM is available or not.
    #[must_use]
    pub fn is_sirm_available(self) -> bool {
        self.0.is_set(0)
    }

    /// Indicate whether EIRM is available or not.
    #[must_use]
    pub fn is_eirm_available(self) -> bool {
        self.0.is_set(1)
    }

    /// Indicate whether IIDC2 is available or not.
    #[must_use]
    pub fn is_iidc2_available(self) -> bool {
        self.0.is_set(2)
    }
}

/// XML file information.
pub struct GenICamFileInfo(pub u32);

impl GenICamFileInfo {
    /// Type of the XML file.
    pub fn file_type(&self) -> Result<GenICamFileType> {
        let raw = self.0 & 0b111;
        match raw {
            0 => Ok(GenICamFileType::DeviceXml),
            1 => Ok(GenICamFileType::BufferXml),
            _ => Err(Error::InvalidDevice(
                format!("invalid U3V GenICamFileType value: {}", raw).into(),
            )),
        }
    }

    /// Compression type of the XML File.
    pub fn compression_type(&self) -> Result<CompressionType> {
        let raw = (self.0 >> 10_i32) & 0b11_1111;
        match raw {
            0 => Ok(CompressionType::Uncompressed),
            1 => Ok(CompressionType::Zip),
            _ => Err(Error::InvalidDevice(
                format!("invalid U3V GenICamFilFormat value: {}", raw).into(),
            )),
        }
    }

    /// `GenICam` schema version of the XML file compiles with.
    #[must_use]
    pub fn schema_version(&self) -> semver::Version {
        let major = (self.0 >> 24_i32) & 0xff;
        let minor = (self.0 >> 16_i32) & 0xff;
        semver::Version::new(u64::from(major), u64::from(minor), 0)
    }
}
