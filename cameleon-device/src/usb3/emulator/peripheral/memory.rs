use std::borrow::Cow;
use std::io::Write;

use byteorder::{WriteBytesExt, LE};
use semver::Version;

use super::{EmulatorError, EmulatorResult};

const SBRM_ADDRESS: u64 = 0xffff;

/// offset | value | Description.
///      0 |     1 | User Defined Name is supported.
///      1 |     0 | Access Privilege and Heartbeat are NOT supported.
///      2 |     0 | Message Channel is NOT supported.
///      3 |     1 | Timestampl is supported.
///    7-4 |  0000 | String Encoding (Ascii).
///      8 |     1 | Family Name is supported.
///      9 |     1 | SBRM is supported.
///     10 |     1 | Endianess Register is supported.
///     11 |     1 | Written Length Field is supported.
///     12 |     1 | Multi Event is supported.
///     13 |     1 | Stacked Commands is supported.
///     14 |     1 | Device Software Interface Version is supported.
///  63-15 |     0 | Reserved. All remained bits are set to 0.
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

macro_rules! string_setter {
    ($fn_name:ident, $prop:ident) => {
        pub fn $fn_name(&mut self, name: &str) -> EmulatorResult<()> {
            verify_str(name)?;
            self.$prop = name.to_owned().into();
            Ok(())
        }
    };
}

impl ABRM {
    string_setter!(set_model_name, model_name);
    string_setter!(set_family_name, family_name);
    string_setter!(set_device_version, device_version);
    string_setter!(set_manufacturer_info, manufacturer_info);
    string_setter!(set_serial_number, serial_number);
    string_setter!(set_user_defined_name, user_defined_name);

    pub(super) fn flush(&self, mut memory: impl Write) -> EmulatorResult<()> {
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

fn verify_str(s: &str) -> EmulatorResult<()> {
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

fn write_str(w: &mut impl Write, s: &str) -> EmulatorResult<()> {
    verify_str(s)?;
    w.write_all(s.as_bytes())?;
    Ok(w.write_u8(0)?) // 0 terminate.
}
