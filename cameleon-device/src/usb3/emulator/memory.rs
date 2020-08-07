use std::borrow::Cow;
use std::io::Write;

use byteorder::{WriteBytesExt, LE};
use semver::Version;

use super::{EmulatorError, Result};

pub struct Memory(Vec<u8>);

impl Memory {
    pub(super) fn new(size: usize) -> Self {
        let raw = vec![0; size];
        Self(raw)
    }

    pub(super) fn from_raw(raw: Vec<u8>) -> Self {
        Self(raw)
    }

    pub(super) fn dump(&self) -> &[u8] {
        &self.0
    }

    pub(super) fn read(&mut self, address: u64) -> Result<u8> {
        let address = address as usize;
        self.verify_address(address)?;
        Ok(self.0[address])
    }

    pub(super) fn write(&mut self, address: u64, data: u8) -> Result<()> {
        let address = address as usize;
        self.verify_address(address)?;

        self.0[address] = data;
        Ok(())
    }

    fn verify_address(&self, address: usize) -> Result<()> {
        if self.0.len() < address {
            Err(EmulatorError::InvalidAddress)
        } else {
            Ok(())
        }
    }
}

const SBRM_ADDRESS: u64 = 0xffff;

/// offset | value | Description.
///      0 |     1 | User Defined Name is supported.
///      1 |     0 | Access Privilege and Heartbeat are NOT supported.
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
    fn flush(&self, mut memory: impl Write) -> Result<()> {
        memory.write_u16::<LE>(self.gen_cp_version.minor as u16)?;
        memory.write_u16::<LE>(self.gen_cp_version.major as u16)?;
        write_str(&mut memory, &self.model_name)?;
        write_str(&mut memory, &self.family_name)?;
        write_str(&mut memory, &self.device_version)?;
        write_str(&mut memory, &self.manufacturer_info)?;
        write_str(&mut memory, &self.serial_number)?;
        write_str(&mut memory, &self.user_defined_name)?;
        memory.write_u64::<LE>(self.device_capability)?;
        memory.write_u32::<LE>(self.maximum_device_response_time)?;
        memory.write_u64::<LE>(self.manifest_table_address)?;
        memory.write_u64::<LE>(self.sbrm_address)?;
        memory.write_u64::<LE>(self.device_configuration)?;
        memory.write_u32::<LE>(self.heartbeat_timeout)?;
        memory.write_u32::<LE>(self.message_channel_id)?;
        memory.write_u64::<LE>(self.timestamp)?;
        memory.write_u32::<LE>(self.timestamp_latch)?;
        memory.write_u64::<LE>(self.timestamp_increment)?;
        memory.write_u32::<LE>(self.access_privilege)?;
        memory.write_u32::<LE>(self.protocol_endianess)?;
        memory.write_u32::<LE>(self.implementation_endianess)?;
        write_str(&mut memory, self.device_software_interface_version)?;

        Ok(())
    }
}

impl Default for ABRM {
    fn default() -> Self {
        Self {
            gen_cp_version: Version::new(1, 3, 0),
            manufacturer_name: "cameleon".into(),
            model_name: "cameleon model".into(),
            family_name: "cameleon family".into(),
            device_version: "none".into(),
            manufacturer_info: "".into(),
            serial_number: "0000".into(),
            user_defined_name: "none".into(),
            device_capability: DEVICE_CAPABILITY,
            maximum_device_response_time: 100,
            manifest_table_address: 0, // TODO: Define manifest table address.
            sbrm_address: SBRM_ADDRESS,
            device_configuration: 0b00,
            heartbeat_timeout: 0,
            message_channel_id: 0,
            timestamp: 0,
            timestamp_latch: 0,
            timestamp_increment: 1000, // Dummy value indicating device clock runs at 1MHz.
            access_privilege: 0,
            protocol_endianess: 0xffff,       // Little endian.
            implementation_endianess: 0xffff, // Little endian.
            device_software_interface_version: "1.0.0",
        }
    }
}

fn verify_str(s: &str) -> Result<()> {
    const STRING_LENGTH_LIMIT: usize = 64;

    if !s.is_ascii() {
        return Err(EmulatorError::InvalidString("string format is not ascii"));
    }

    // String in register must be 0 terminated.
    if s.as_bytes().len() > STRING_LENGTH_LIMIT - 1 {
        return Err(EmulatorError::InvalidString("string is too long."));
    }

    Ok(())
}

fn write_str(w: &mut impl Write, s: &str) -> Result<()> {
    verify_str(s)?;
    w.write(s.as_bytes())?;
    Ok(w.write_u8(0)?) // 0 terminate.
}
