use std::{fmt, time::Duration};

use cameleon_device::u3v::register_map::*;
use semver;

use crate::device::{DeviceError, DeviceResult};

use super::control_handle::ControlHandle;

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

impl Abrm {
    pub(super) fn new(handle: &ControlHandle) -> DeviceResult<Self> {
        use abrm::*;

        let mut buf = vec![0; 64];
        macro_rules! sync_register {
            ($register_info:ident, $ty:ty) => {{
                let (addr, len) = $register_info;
                let len = len as usize;
                if buf.len() < len {
                    buf.resize(len, 0);
                }

                handle.read_mem(addr, &mut buf[..len])?;
                <$ty>::parse_bytes(&buf[..len])?
            }};
        }

        let device_capability = sync_register!(DEVICE_CAPABILITY, DeviceCapability);
        let gencp_version = sync_register!(GENCP_VERSION, semver::Version);
        let manufacturer_name = sync_register!(GENCP_VERSION, String);
        let model_name = sync_register!(MODEL_NAME, String);
        let family_name = if device_capability.is_family_name_supported() {
            Some(sync_register!(FAMILY_NAME, String))
        } else {
            None
        };
        let device_version = sync_register!(DEVICE_VERSION, semver::Version);
        let manufacturer_info = sync_register!(MANUFACTURER_INFO, String);
        let serial_number = sync_register!(SERIAL_NUMBER, String);
        let user_defined_name = if device_capability.is_user_defined_name_suported() {
            Some(sync_register!(USER_DEFINED_NAME, String))
        } else {
            None
        };
        let maximum_device_response_time = sync_register!(MAXIMUM_DEVICE_RESPONSE_TIME, Duration);
        let manifest_table_address = sync_register!(MANIFEST_TABLE_ADDRESS, u64);
        let sbrm_address = sync_register!(SBRM_ADDRESS, u64);
        let timestamp = sync_register!(TIMESTAMP, u64);
        let timestamp_increment = sync_register!(TIMESTAMP_INCREMENT, u32);
        let device_software_interface_version =
            if device_capability.is_device_software_interface_version_supported() {
                Some(sync_register!(DEVICE_SOFTWARE_INTERFACE_VERSION, String))
            } else {
                None
            };

        Ok(Self {
            gencp_version,
            manufacturer_name,
            model_name,
            family_name,
            device_version,
            manufacturer_info,
            serial_number,
            user_defined_name,
            device_capability,
            maximum_device_response_time,
            manifest_table_address,
            sbrm_address,
            timestamp,
            timestamp_increment,
            device_software_interface_version,
        })
    }
}

struct DeviceCapability {
    raw: [u8; 8],
}

impl DeviceCapability {
    fn is_user_defined_name_suported(&self) -> bool {
        self.is_bit_set(0)
    }

    fn is_family_name_supported(&self) -> bool {
        self.is_bit_set(8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    fn is_multi_event_supported(&self) -> bool {
        self.is_bit_set(12)
    }

    /// Indicate whether the device supports stacked commands (ReadMemStacked and WriteMemStacked).
    fn is_stacked_commands_supported(&self) -> bool {
        self.is_bit_set(13)
    }

    /// Indicate whether the device supports software interface version is supported.
    fn is_device_software_interface_version_supported(&self) -> bool {
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

trait ParseBytes: Sized {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self>;
}

impl ParseBytes for semver::Version {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}

impl ParseBytes for String {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}

impl ParseBytes for Duration {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}

impl ParseBytes for u64 {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}

impl ParseBytes for u32 {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        todo!()
    }
}
