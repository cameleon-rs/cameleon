/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::bit_op::BitOp;

use crate::{CharacterEncoding, Endianness};

/// (Address, Length) of registers of Bootstrap Register Map.
pub mod bootstrap {
    pub const VERSION: (u32, u16) = (0x0000, 4);
    pub const DEVICE_MODE: (u32, u16) = (0x0004, 4);
    pub const DEVICE_MAC_ADDRESS_HIGH_0: (u32, u16) = (0x0008, 4);
    pub const DEVICE_MAC_ADDRESS_LOW_0: (u32, u16) = (0x000C, 4);
    pub const NETWORK_INTERFACE_CAPABILITY_0: (u32, u16) = (0x0010, 4);
    pub const NETWORK_INTERFACE_CONFIGURATION_0: (u32, u16) = (0x0014, 4);
    pub const CURRENT_IP_ADDRESS_0: (u32, u16) = (0x0024, 4);
    pub const CURRENT_SUBNET_MASK_0: (u32, u16) = (0x0034, 4);
    pub const CURRENT_DEFAULT_GATEWAY_0: (u32, u16) = (0x0044, 4);
    pub const MANUFACTURER_NAME: (u32, u16) = (0x0048, 32);
    pub const MODEL_NAME: (u32, u16) = (0x0068, 32);
    pub const DEVICE_VERSION: (u32, u16) = (0x0088, 32);
    pub const MANUFACTURER_INFO: (u32, u16) = (0x00A8, 48);
    pub const SERIAL_NUMBER: (u32, u16) = (0x00D8, 16);
    pub const USER_DEFINED_NAME: (u32, u16) = (0x00E8, 16);
    pub const FIRST_URL: (u32, u16) = (0x0200, 512);
    pub const SECOND_URL: (u32, u16) = (0x0400, 512);
    pub const NUMBER_OF_NETWORK_INTERFACES: (u32, u16) = (0x0600, 4);
    pub const PERSISTENT_IP_ADDRESS_0: (u32, u16) = (0x064C, 4);
    pub const PERSISTENT_SUBNET_MASK_0: (u32, u16) = (0x065C, 4);
    pub const PERSISTENT_DEFAULT_GATEWAY_0: (u32, u16) = (0x066C, 4);
    pub const LINK_SPEED_0: (u32, u16) = (0x0670, 4);
    pub const DEVICE_MAC_ADDRESS_HIGH_1: (u32, u16) = (0x0680, 4);
    pub const DEVICE_MAC_ADDRESS_LOW_1: (u32, u16) = (0x0684, 4);
    pub const NETWORK_INTERFACE_CAPABILITY_1: (u32, u16) = (0x0688, 4);
    pub const NETWORK_INTERFACE_CONFIGURATION_1: (u32, u16) = (0x068C, 4);
    pub const CURRENT_IP_ADDRESS_1: (u32, u16) = (0x069C, 4);
    pub const CURRENT_SUBNET_MASK_1: (u32, u16) = (0x06AC, 4);
    pub const CURRENT_DEFAULT_GATEWAY_1: (u32, u16) = (0x06BC, 4);
    pub const PERSISTENT_IP_ADDRESS_1: (u32, u16) = (0x06CC, 4);
    pub const PERSISTENT_SUBNET_MASK_1: (u32, u16) = (0x06DC, 4);
    pub const PERSISTENT_DEFAULT_GATEWAY_1: (u32, u16) = (0x06EC, 4);
    pub const LINK_SPEED_1: (u32, u16) = (0x06F0, 4);
    pub const DEVICE_MAC_ADDRESS_HIGH_2: (u32, u16) = (0x0700, 4);
    pub const DEVICE_MAC_ADDRESS_LOW_2: (u32, u16) = (0x0704, 4);
    pub const NETWORK_INTERFACE_CAPABILITY_2: (u32, u16) = (0x0708, 4);
    pub const NETWORK_INTERFACE_CONFIGURATION_2: (u32, u16) = (0x070C, 4);
    pub const CURRENT_IP_ADDRESS_2: (u32, u16) = (0x071C, 4);
    pub const CURRENT_SUBNET_MASK_2: (u32, u16) = (0x072C, 4);
    pub const CURRENT_DEFAULT_GATEWAY_2: (u32, u16) = (0x073C, 4);
    pub const PERSISTENT_IP_ADDRESS_2: (u32, u16) = (0x074C, 4);
    pub const PERSISTENT_SUBNET_MASK_2: (u32, u16) = (0x075C, 4);
    pub const PERSISTENT_DEFAULT_GATEWAY_2: (u32, u16) = (0x076C, 4);
    pub const LINK_SPEED_2: (u32, u16) = (0x0770, 4);
    pub const DEVICE_MAC_ADDRESS_HIGH_3: (u32, u16) = (0x0780, 4);
    pub const DEVICE_MAC_ADDRESS_LOW_3: (u32, u16) = (0x0784, 4);
    pub const NETWORK_INTERFACE_CAPABILITY_3: (u32, u16) = (0x0788, 4);
    pub const NETWORK_INTERFACE_CONFIGURATION_3: (u32, u16) = (0x078C, 4);
    pub const CURRENT_IP_ADDRESS_3: (u32, u16) = (0x079C, 4);
    pub const CURRENT_SUBNET_MASK_3: (u32, u16) = (0x07AC, 4);
    pub const CURRENT_DEFAULT_GATEWAY_3: (u32, u16) = (0x07BC, 4);
    pub const PERSISTENT_IP_ADDRESS_3: (u32, u16) = (0x07CC, 4);
    pub const PERSISTENT_SUBNET_MASK_3: (u32, u16) = (0x07DC, 4);
    pub const PERSISTENT_DEFAULT_GATEWAY_3: (u32, u16) = (0x07EC, 4);
    pub const LINK_SPEED_3: (u32, u16) = (0x07F0, 4);
    pub const NUMBER_OF_MESSAGE_CHANNELS: (u32, u16) = (0x0900, 4);
    pub const NUMBER_OF_STREAM_CHANNELS: (u32, u16) = (0x0904, 4);
    pub const NUMBER_OF_ACTION_SIGNALS: (u32, u16) = (0x0908, 4);
    pub const ACTION_DEVICE_KEY: (u32, u16) = (0x090C, 4);
    pub const NUMBER_OF_ACTIVE_LINKS: (u32, u16) = (0x0910, 4);
    pub const GVSP_CAPABILITY: (u32, u16) = (0x092c, 4);
    pub const MESSAGE_CHANNEL_CAPABILITY: (u32, u16) = (0x0930, 4);
    pub const GVCP_CAPABILITY: (u32, u16) = (0x0934, 4);
    pub const HEARTBEAT_TIMEOUT: (u32, u16) = (0x0938, 4);
    pub const TIMESTAMP_TICK_FREQUENCY_HIGH: (u32, u16) = (0x093C, 4);
    pub const TIMESTAMP_TICK_FREQUENCY_LOW: (u32, u16) = (0x0940, 4);
    pub const TIMESTAMP_CONTROL: (u32, u16) = (0x0944, 4);
    pub const TIMESTAMP_VALUE_HIGH: (u32, u16) = (0x0948, 4);
    pub const TIMESTAMP_VALUE_LOW: (u32, u16) = (0x094C, 4);
    pub const DISCOVERY_ACK_DELAY: (u32, u16) = (0x0950, 4);
    pub const GVCP_CONFIGURATION: (u32, u16) = (0x0954, 4);
    pub const PENDING_TIMEOUT: (u32, u16) = (0x0958, 4);
    pub const CONTROL_SWITCHOVER_KEY: (u32, u16) = (0x095C, 4);
    pub const GVSP_CONFIGURATION: (u32, u16) = (0x0960, 4);
    pub const PHYSICAL_LINK_CONFIGURATION_CAPABILITY: (u32, u16) = (0x0964, 4);
    pub const PHYSICAL_LINK_CONFIGURATION: (u32, u16) = (0x0968, 4);
    pub const IEEE_1588_STATUS: (u32, u16) = (0x096C, 4);
    pub const SCHEDULED_ACTION_COMMAND_QUEUE_SIZE: (u32, u16) = (0x0970, 4);
    pub const CONTROL_CHANNEL_PRIVILEDGE: (u32, u16) = (0x0A00, 4);
    pub const PRIMARY_APPLICATION_PORT: (u32, u16) = (0x0A04, 4);
    pub const PRIMARY_APPLICATION_IP_ADDRESS: (u32, u16) = (0x0A14, 4);
    pub const MANIFEST_TABLE: (u32, u16) = (0x9000, 512);
}

/// (Offset, Length) of registers of Stream Register map.
pub mod stream {
    pub const STREAM_CHANNEL_PORT: (u32, u16) = (0x0000, 4);
    pub const STREAM_CHANNEL_PACKET_SIZE: (u32, u16) = (0x0004, 4);
    pub const STREAM_CHANNEL_PACKET_DELAY: (u32, u16) = (0x0008, 4);
    pub const STREAM_CHANNEL_DESTINATION_ADDRESS: (u32, u16) = (0x0018, 4);
    pub const STREAM_CHANNEL_SOURCE_PORT: (u32, u16) = (0x001C, 4);
    pub const STREAM_CHANNEL_CAPABILITY: (u32, u16) = (0x0020, 4);
    pub const STREAM_CHANNEL_CONFIGURATION: (u32, u16) = (0x0024, 4);
    pub const STREAM_CHANNEL_ZONE: (u32, u16) = (0x0028, 4);
    pub const STREAM_CHANNEL_ZONE_DIRECTION: (u32, u16) = (0x002C, 4);

    pub fn base_address(channel_index: u32) -> u32 {
        0x0D00 + 0x0040 * channel_index
    }
}

pub mod message {
    pub const MESSAGE_CHANNEL_PORT: (u32, u16) = (0x0B00, 4);
    pub const MESSAGE_CHANNEL_DESTINATION: (u32, u16) = (0x0B10, 4);
    pub const MESSAGE_CHANNEL_TRANSMISSION_TIMEOUT: (u32, u16) = (0x0B14, 4);
    pub const MESSAGE_CHANNEL_RETRY_COUNT: (u32, u16) = (0x0B18, 4);
    pub const MESSAGE_CHANNEL_SOURCE_PORT: (u32, u16) = (0x0B1C, 4);
}

/// (Offset, Length) of registers of ActionGroup Register map.
pub mod action_group {
    pub const ACTION_GROUP_KEY: (u32, u16) = (0x0000, 4);
    pub const ACTION_GROUP_MASK: (u32, u16) = (0x0004, 4);

    pub fn base_address(action_index: u32) -> u32 {
        0x9800 + 0x0010 * action_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceMode(u32);

impl DeviceMode {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn endianness(self) -> Endianness {
        if self.0 >> 31_u8 == 0 {
            Endianness::LE
        } else {
            Endianness::BE
        }
    }

    pub fn device_class(self) -> DeviceClass {
        let code = (self.0 >> 28_u8) & 0b111;
        match code {
            0 => DeviceClass::Transmitter,
            1 => DeviceClass::Receiver,
            2 => DeviceClass::Transceiver,
            3 => DeviceClass::Peripheral,
            _ => unreachable!(),
        }
    }

    pub fn link_configuration(self) -> LinkConfiguration {
        let code = (self.0 >> 24_u8) & 0b1111;
        match code {
            0 => LinkConfiguration::SingleLink,
            1 => LinkConfiguration::MultipleLink,
            2 => LinkConfiguration::StaticLAG,
            3 => LinkConfiguration::DynamicLAG,
            _ => unreachable!(),
        }
    }

    pub fn char_encoding(self) -> CharacterEncoding {
        let code = self.0 & 0b1111_1111;
        match code {
            1 => CharacterEncoding::Utf8,
            0 | 2 => CharacterEncoding::Ascii,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Transmitter,
    Receiver,
    Transceiver,
    Peripheral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkConfiguration {
    SingleLink,
    MultipleLink,
    StaticLAG,
    DynamicLAG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicCapability(u32);

impl NicCapability {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_pause_reception_supported(self) -> bool {
        self.0.is_set(0)
    }

    pub fn is_pause_generation_supported(self) -> bool {
        self.0.is_set(1)
    }

    pub fn is_link_local_address_supported(self) -> bool {
        self.0.is_set(29)
    }

    pub fn is_dhcp_supported(self) -> bool {
        self.0.is_set(30)
    }

    pub fn is_force_ip_supported(self) -> bool {
        self.0.is_set(31)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicConfiguration(u32);

impl NicConfiguration {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_pause_reception_enabled(self) -> bool {
        self.0.is_set(0)
    }

    pub fn enable_pause_reception(self) -> Self {
        Self(self.0.set_bit(0))
    }

    pub fn disable_pause_reception(self) -> Self {
        Self(self.0.set_bit(0))
    }

    pub fn is_pause_generation_enabled(self) -> bool {
        self.0.is_set(1)
    }

    pub fn enable_pause_generation(self) -> Self {
        Self(self.0.set_bit(1))
    }

    pub fn disable_pause_generation(self) -> Self {
        Self(self.0.clear_bit(1))
    }

    pub fn is_link_local_address_enabled(self) -> bool {
        self.0.is_set(29)
    }

    pub fn is_dhcp_enabled(self) -> bool {
        self.0.is_set(30)
    }

    pub fn is_force_ip_enabled(self) -> bool {
        self.0.is_set(31)
    }

    pub fn enable_force_ip(self) -> Self {
        Self(self.0.set_bit(31))
    }

    pub fn disable_force_ip(self) -> Self {
        Self(self.0.clear_bit(31))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlChannelPriviledge(u32);

impl ControlChannelPriviledge {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn switchover_key(self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub fn set_switchover_key(self, key: u16) -> Self {
        let key = key as u32;
        Self((self.0.overflowing_shl(16).0 >> 16) | (key << 16))
    }

    pub fn enable_switchover(self) -> Self {
        Self(self.0.set_bit(29))
    }

    pub fn disable_switchover(self) -> Self {
        Self(self.0.clear_bit(29))
    }

    pub fn is_switchover_enabled(self) -> bool {
        self.0.is_set(29)
    }

    pub fn enable_control_access(self) -> Self {
        Self(self.0.set_bit(30))
    }

    pub fn disable_control_access(self) -> Self {
        Self(self.0.clear_bit(30))
    }

    pub fn is_control_access_enabled(self) -> bool {
        self.0.is_set(30)
    }

    pub fn enable_exclusive_access(self) -> Self {
        Self(self.0.set_bit(31))
    }

    pub fn disable_exclusive_access(self) -> Self {
        Self(self.0.clear_bit(31))
    }

    pub fn is_exclusive_access_enabled(self) -> bool {
        self.0.is_set(31)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GvcpCapability(u32);

impl GvcpCapability {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_user_defined_name_supported(self) -> bool {
        self.0.is_set(0)
    }

    pub fn is_serial_number_supported(self) -> bool {
        self.0.is_set(1)
    }

    pub fn is_heartbeat_disable_supported(self) -> bool {
        self.0.is_set(2)
    }

    pub fn is_linkspeed_register_supported(self) -> bool {
        self.0.is_set(3)
    }

    pub fn is_manifest_table_supported(self) -> bool {
        self.0.is_set(5)
    }

    pub fn is_switchover_supported(self) -> bool {
        self.0.is_set(10)
    }

    pub fn is_action_supported(self) -> bool {
        self.0.is_set(25)
    }

    pub fn is_pending_ack_supported(self) -> bool {
        self.0.is_set(26)
    }

    pub fn is_event_data_supported(self) -> bool {
        self.0.is_set(27)
    }

    pub fn is_event_supported(self) -> bool {
        self.0.is_set(28)
    }

    pub fn is_packet_resend_supported(self) -> bool {
        self.0.is_set(29)
    }

    pub fn is_write_mem_supported(self) -> bool {
        self.0.is_set(30)
    }

    pub fn is_multiple_register_access_supported(self) -> bool {
        self.0.is_set(31)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GvspCapability(u32);

impl GvspCapability {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_scsp_register_supported(self) -> bool {
        self.0.is_set(0)
    }

    pub fn is_16bit_block_id_supported(self) -> bool {
        self.0.is_set(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageChannelCapability(u32);

impl MessageChannelCapability {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_mcsp_register_supported(self) -> bool {
        self.0.is_set(0)
    }
}
