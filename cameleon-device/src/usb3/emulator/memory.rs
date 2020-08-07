use std::borrow::Cow;
use std::io::Write;

use super::{EmulatorError, Result};
use semver::Version;

pub struct Memory {
    raw: Vec<u8>,
}

impl Memory {
    pub(super) fn new(size: usize) -> Self {
        let raw = vec![0; size];
        let mut memory = Self { raw };
        memory
    }

    pub(super) fn from_raw(raw: Vec<u8>) -> Self {
        Self { raw }
    }

    pub(super) fn dump(&self) -> &[u8] {
        &self.raw
    }

    pub(super) fn read(&mut self, address: u64) -> Result<u8> {
        let address = address as usize;
        self.check_access(address)?;
        Ok(self.raw[address])
    }

    pub(super) fn write(&mut self, address: u64, data: u8) -> Result<()> {
        let address = address as usize;
        self.check_access(address)?;

        self.raw[address] = data;
        Ok(())
    }

    fn check_access(&self, address: usize) -> Result<()> {
        if self.raw.len() < address {
            Err(EmulatorError::MemoryAccessViolation)
        } else {
            Ok(())
        }
    }
}

const SBRM_ADDRESS: u64 = 0xffff;

/// offset | value | Description.
///      0 |     1 | User Defined Name is supported.
///      1 |     0 | Access Priviledge and Heartbeat are NOT supported.
///      2 |     0 | Message Channel is NOT supported.
///      3 |     1 | Timestampl is supported.
///      4 |  0000 | String Encoding (Ascii).
///      8 |     1 | Family Name is supported.
///      9 |     1 | SBRM is supported.
///     10 |     1 | Endianess Register is supported.
///     11 |     1 | Written Length Field is supported.
///     12 |     1 | Multi Event is supported.
///     13 |     1 | Stacked Commands is supported.
///     14 |     1 | Device Software Interface Version is supported.
///     15 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CAPABILITY: u64 = 0b111111100001001;

#[derive(Clone)]
pub struct ABRM {
    gen_cp_version: Version,
    manufacturer_name: Cow<'static, str>,
    model_name: Cow<'static, str>,
    family_name: Cow<'static, str>,
    device_version: Cow<'static, str>,
    manufacturer_info: Cow<'static, str>,
    serial_number: Cow<'static, str>,
    user_defined_name: Cow<'static, str>,
    device_capability: u64,
    maximum_device_response_time: u32,
    manifest_table_address: u64,
    sbrm_address: u64,
    device_configuration: u64,
    heartbeat_timeout: u32,
    message_channel_id: u32,
    timestamp: u64,
    timestamp_latch: u32,
    timestamp_increment: u64,
    access_privilege: u32,
    protocol_endianess: u32,
    implementation_endianess: u32,
    device_software_interface_version: &'static str,
}

impl ABRM {
    fn flush(&self, memory: impl Write) -> Result<()> {
        todo!();
    }
}

impl Default for ABRM {
    fn default() -> Self {
        Self {
            gen_cp_version: Version::new(1, 2, 0),
            manufacturer_name: "cameleon".into(),
            model_name: "cameleon model".into(),
            family_name: "cameleon family".into(),
            device_version: "none".into(),
            manufacturer_info: "".into(),
            serial_number: "0000".into(),
            user_defined_name: "none".into(),
            device_capability: DEVICE_CAPABILITY,
            maximum_device_response_time: 100,
            manifest_table_address: 0, // TODO: Define manifest address,
            sbrm_address: SBRM_ADDRESS,
            device_configuration: 0b00,
            heartbeat_timeout: 0,
            message_channel_id: 0,
            timestamp: 0,
            timestamp_latch: 0,
            timestamp_increment: 1000, // Dummy value indicating device clock runs at 1MHz.
            access_privilege: 0,
            protocol_endianess: 0xffff,       // Big endian.
            implementation_endianess: 0xffff, // Big endian.
            device_software_interface_version: "1.0.0",
        }
    }
}
