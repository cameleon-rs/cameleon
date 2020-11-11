use std::{fmt, time::Duration};

use semver;

#[derive(Debug)]
struct Abrm {
    gencp_version: semver::Version,
    manufacturer_name: String,
    model_name: String,
    family_name: Option<String>,
    device_version: semver::Version,
    manufacturer_info: String,
    serial_number: String,
    user_defined_name: Option<String>,
    device_capability: DeviceCapability,
    maximum_device_response_time: Duration,
    manifest_table_address: u64,
    sbrm_address: u64,
    timestamp: u64,
    timestamp_latch: u32,
    timestamp_increment: u32,
    device_software_interface_version: Option<String>,
    // NOTE. U3V devices doesn't use Below registers even though GenCP defines it.
    // * DEVICE_CONFIGURATION
    // * HEARTBEAT_TIMEOUT
    // * MESSAGE_CHANNEL_ID
    // * ACCESS_PRIVILEGE
    // * PROTOCOL_ENDIANNESS
    // * IMPLEMENTATION_ENDIANNESS
}

pub struct DeviceCapability {
    raw: [u8; 8],
}

impl DeviceCapability {
    pub fn is_user_defined_name_suported(&self) -> bool {
        self.is_bit_set(0)
    }

    pub fn is_family_name_supported(&self) -> bool {
        self.is_bit_set(8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    pub fn is_multi_event_supported(&self) -> bool {
        self.is_bit_set(12)
    }

    /// Indicate whether the device supports stacked commands (ReadMemStacked and WriteMemStacked).
    pub fn is_stacked_commands_supported(&self) -> bool {
        self.is_bit_set(13)
    }

    /// Indicate whether the device supports software interface version is supported.
    pub fn is_device_software_interface_version_supported(&self) -> bool {
        self.is_bit_set(14)
    }

    fn is_bit_set(&self, offset: usize) -> bool {
        debug_assert!(offset < 64);

        let idx = offset / 8;
        let rem = offset % 8;
        (self.raw[idx] >> rem) & 1 == 1
    }
}

impl fmt::Debug for DeviceCapability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeviceCapability")
            .field(
                "is_user_defined_name_suported",
                &format_args!("{}", self.is_user_defined_name_suported()),
            )
            .field(
                "is_family_name_supported",
                &format_args!("{}", self.is_family_name_supported()),
            )
            .field(
                "is_multi_event_supported",
                &format_args!("{}", self.is_multi_event_supported()),
            )
            .field(
                "is_stacked_commands_supported",
                &format_args!("{}", self.is_stacked_commands_supported()),
            )
            .field(
                "is_device_software_interface_version_supported",
                &format_args!("{}", self.is_device_software_interface_version_supported()),
            )
            .finish()
    }
}
