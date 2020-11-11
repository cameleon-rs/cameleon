use std::time::Duration;

use semver;

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
    sbrm_address: Option<u64>,
    timestamp: Option<u64>,
    timestamp_latch: Option<u32>,
    timestamp_increment: Option<u32>,
    device_software_interface_version: Option<String>,
    // NOTE. U3V devices doesn't use Below registers even though GenCP defines it.
    // * DEVICE_CONFIGURATION
    // * HEARTBEAT_TIMEOUT
    // * MESSAGE_CHANNEL_ID
    // * ACCESS_PRIVILEGE
    // * PROTOCOL_ENDIANNESS
    // * IMPLEMENTATION_ENDIANNESS
}

struct DeviceCapability {
    raw: u64,
}
