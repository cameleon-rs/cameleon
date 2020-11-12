use std::{convert::TryInto, fmt, time::Duration};

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
    device_version: String,
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

/// Read and parse register value.
fn read_register<T>(
    handle: &ControlHandle,
    buf: &mut Vec<u8>,
    addr: u64,
    len: u16,
) -> DeviceResult<T>
where
    T: ParseBytes,
{
    let len = len as usize;
    if buf.len() < len {
        buf.resize(len, 0);
    }

    handle.read_mem(addr, &mut buf[..len])?;
    T::parse_bytes(&buf[..len])
}

impl Abrm {
    pub(super) fn new(handle: &ControlHandle) -> DeviceResult<Self> {
        use abrm::*;

        let mut buf = vec![0; 64];

        let device_capability: DeviceCapability =
            read_register(handle, &mut buf, DEVICE_CAPABILITY.0, DEVICE_CAPABILITY.1)?;

        let gencp_version = read_register(handle, &mut buf, GENCP_VERSION.0, GENCP_VERSION.1)?;
        let manufacturer_name =
            read_register(handle, &mut buf, MANUFACTURER_NAME.0, MANUFACTURER_NAME.1)?;
        let model_name = read_register(handle, &mut buf, MODEL_NAME.0, MODEL_NAME.1)?;
        let family_name = if device_capability.is_family_name_supported() {
            Some(read_register(
                handle,
                &mut buf,
                FAMILY_NAME.0,
                FAMILY_NAME.1,
            )?)
        } else {
            None
        };
        let device_version = read_register(handle, &mut buf, DEVICE_VERSION.0, DEVICE_VERSION.1)?;
        let manufacturer_info =
            read_register(handle, &mut buf, MANUFACTURER_INFO.0, MANUFACTURER_INFO.1)?;
        let serial_number = read_register(handle, &mut buf, SERIAL_NUMBER.0, SERIAL_NUMBER.1)?;
        let user_defined_name = if device_capability.is_user_defined_name_suported() {
            Some(read_register(
                handle,
                &mut buf,
                USER_DEFINED_NAME.0,
                USER_DEFINED_NAME.1,
            )?)
        } else {
            None
        };
        let maximum_device_response_time = read_register(
            handle,
            &mut buf,
            MAXIMUM_DEVICE_RESPONSE_TIME.0,
            MAXIMUM_DEVICE_RESPONSE_TIME.1,
        )?;
        let manifest_table_address = read_register(
            handle,
            &mut buf,
            MANIFEST_TABLE_ADDRESS.0,
            MANIFEST_TABLE_ADDRESS.1,
        )?;
        let sbrm_address = read_register(handle, &mut buf, SBRM_ADDRESS.0, SBRM_ADDRESS.1)?;
        let timestamp = read_register(handle, &mut buf, TIMESTAMP.0, TIMESTAMP.1)?;
        let timestamp_increment = read_register(
            handle,
            &mut buf,
            TIMESTAMP_INCREMENT.0,
            TIMESTAMP_INCREMENT.1,
        )?;
        let device_software_interface_version =
            if device_capability.is_device_software_interface_version_supported() {
                Some(read_register(
                    handle,
                    &mut buf,
                    DEVICE_SOFTWARE_INTERFACE_VERSION.0,
                    TIMESTAMP.1,
                )?)
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
        let minor = u16::parse_bytes(&bytes[0..16])?;
        let major = u16::parse_bytes(&bytes[16..])?;
        Ok(semver::Version::new(major as u64, minor as u64, 0))
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        let raw = bytes.try_into().unwrap();
        Ok(Self { raw })
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
        let raw = u64::parse_bytes(bytes)?;
        Ok(Duration::from_millis(raw))
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
