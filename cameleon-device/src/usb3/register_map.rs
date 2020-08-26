/// (Address, Length, Access Right) of all entries in Technology Agnostic Boot Register Map (ABRM).
pub mod abrm {
    use super::AccessRight::{self, *};
    pub const GENCP_VERSION: (u64, u16, AccessRight) = (0x0000, 4, RO);
    pub const MANUFACTURER_NAME: (u64, u16, AccessRight) = (0x0004, 64, RO);
    pub const MODEL_NAME: (u64, u16, AccessRight) = (0x0044, 64, RO);
    pub const FAMILY_NAME: (u64, u16, AccessRight) = (0x0084, 64, RO);
    pub const DEVICE_VERSION: (u64, u16, AccessRight) = (0x00C4, 64, RO);
    pub const MANUFACTURER_INFO: (u64, u16, AccessRight) = (0x0104, 64, RO);
    pub const SERIAL_NUMBER: (u64, u16, AccessRight) = (0x0144, 64, RO);
    pub const USER_DEFINED_NAME: (u64, u16, AccessRight) = (0x0184, 64, RW);
    pub const DEVICE_CAPABILITY: (u64, u16, AccessRight) = (0x01C4, 8, RO);
    pub const MAXIMUM_DEVICE_RESPONSE_TIME: (u64, u16, AccessRight) = (0x01CC, 4, RO);
    pub const MANIFEST_TABLE_ADDRESS: (u64, u16, AccessRight) = (0x01D0, 8, RO);
    pub const SBRM_ADDRESS: (u64, u16, AccessRight) = (0x01D8, 8, RO);
    pub const DEVICE_CONFIGURATION: (u64, u16, AccessRight) = (0x01E0, 8, RO);
    pub const HEARTBEAT_TIMEOUT: (u64, u16, AccessRight) = (0x01E8, 4, RW);
    pub const MESSAGE_CHANNEL_ID: (u64, u16, AccessRight) = (0x01EC, 4, RW);
    pub const TIMESTAMP: (u64, u16, AccessRight) = (0x01F0, 8, RO);
    pub const TIMESTAMP_LATCH: (u64, u16, AccessRight) = (0x01F8, 4, WO);
    pub const TIMESTAMP_INCREMENT: (u64, u16, AccessRight) = (0x01FC, 8, RO);
    pub const ACCESS_PRIVILEGE: (u64, u16, AccessRight) = (0x0204, 4, RW);
    pub const PROTOCOL_ENDIANESS: (u64, u16, AccessRight) = (0x0208, 4, RO);
    pub const IMPLEMENTATION_ENDIANESS: (u64, u16, AccessRight) = (0x020C, 4, RO);
    pub const DEVICE_SOFTWARE_INTERFACE_VERSION: (u64, u16, AccessRight) = (0x0210, 64, RO);
}

/// (Offset, Length, Access Right) of entries in Technology Specific Boot Register Map (SBRM).
/// SBRM base address can be obtained by reading `abrm::SBRM_ADDRESS`.
pub mod sbrm {
    use super::AccessRight::{self, *};
    pub const U3V_VERSION: (u64, u16, AccessRight) = (0x0000, 4, RO);
    pub const U3VCP_CAPABILITY_REGISTER: (u64, u16, AccessRight) = (0x0004, 8, RO);
    pub const U3VCP_CONFIGURATION_REGISTER: (u64, u16, AccessRight) = (0x000C, 8, RW);
    pub const MAXIMUM_COMMAND_TRANSFER_LENGTH: (u64, u16, AccessRight) = (0x014, 4, RO);
    pub const MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH: (u64, u16, AccessRight) = (0x018, 4, RO);
    pub const NUMBER_OF_STREAM_CHANNELS: (u64, u16, AccessRight) = (0x01C, 4, RO);
    pub const SIRM_ADDRESS: (u64, u16, AccessRight) = (0x020, 8, RO);
    pub const SIRM_LENGTH: (u64, u16, AccessRight) = (0x028, 4, RO);
    pub const EIRM_ADDRESS: (u64, u16, AccessRight) = (0x02C, 8, RO);
    pub const EIRM_LENGTH: (u64, u16, AccessRight) = (0x034, 4, RO);
    pub const IIDC2_ADDRESS: (u64, u16, AccessRight) = (0x038, 8, RO);
    pub const CURRENT_SPEED: (u64, u16, AccessRight) = (0x040, 4, RO);
}

/// (Offset, Length, Access Right) of all entries in Event Interface Register Map (EIRM).
/// SIRM base address can be obtained by
/// sbrm::EIRM_ADDRESS.
pub mod eirm {
    use super::AccessRight::{self, *};
    pub const EI_CONTROL: (u64, u16, AccessRight) = (0x0000, 4, RW);
    pub const MAXIMUM_EVENT_TRANSFER_LENGTH: (u64, u16, AccessRight) = (0x0004, 4, RO);
    pub const EVENT_TEST_CONTROL: (u64, u16, AccessRight) = (0x0008, 4, RO);
}

/// (Offset, Length, AccessRight) of all entries in Streaming Interface Register Map (SIRM).
/// SIRM base address can be obtained by
/// `sbrm::SIRM_ADDRESS`.
pub mod sirm {
    use super::AccessRight::{self, *};
    pub const SI_INFO: (u64, u16, AccessRight) = (0x0000, 4, RO);
    pub const SI_CONTROL: (u64, u16, AccessRight) = (0x0004, 4, RW);
    pub const REQUIRED_PAYLOAD_SIZE: (u64, u16, AccessRight) = (0x0008, 8, RO);
    pub const REQUIRED_LEADER_SIZE: (u64, u16, AccessRight) = (0x0010, 4, RO);
    pub const REQUIRED_TRAILER_SIZE: (u64, u16, AccessRight) = (0x0014, 4, RO);
    pub const MAXIMUM_LEADER_SIZE: (u64, u16, AccessRight) = (0x0018, 4, RW);
    pub const PAYLOAD_TRANSFER_SIZE: (u64, u16, AccessRight) = (0x001C, 4, RW);
    pub const PAYLOAD_TRANSFER_COUNT: (u64, u16, AccessRight) = (0x0020, 4, RW);
    pub const PAYLOAD_FINAL_TRANSFER1_SIZE: (u64, u16, AccessRight) = (0x0024, 4, RW);
    pub const PAYLOAD_FINAL_TRANSFER2_SIZE: (u64, u16, AccessRight) = (0x0028, 4, RW);
    pub const MAXIMUM_TRAILER_SIZE: (u64, u16, AccessRight) = (0x002C, 4, RW);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum AccessRight {
    /// Not Available.
    NA,

    /// ReadOnly.
    RO,

    /// WriteOnly.
    WO,

    /// ReadWrite.
    RW,
}

impl AccessRight {
    pub fn is_readable(self) -> bool {
        self.as_num() & 0b1 == 1
    }

    pub fn is_writable(self) -> bool {
        self.as_num() >> 1 == 1
    }

    pub(crate) fn as_num(self) -> u8 {
        match self {
            Self::NA => 0b00,
            Self::RO => 0b01,
            Self::WO => 0b10,
            Self::RW => 0b11,
        }
    }
}
