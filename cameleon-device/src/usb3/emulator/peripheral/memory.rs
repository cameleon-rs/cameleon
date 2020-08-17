use std::borrow::Cow;
use std::io::Write;
use std::ops::Range;

use byteorder::{WriteBytesExt, LE};
use semver::Version;

use crate::usb3::register_map::AccessRight;

use super::{EmulatorError, EmulatorResult};

pub(super) struct Memory {
    inner: Vec<u8>,
    protection: MemoryProtection,
}

impl Memory {
    pub(super) fn new(inner: Vec<u8>) -> Self {
        Self {
            protection: MemoryProtection::new(inner.len()),
            inner,
        }
    }

    pub(super) fn read_mem(&self, range: Range<usize>) -> EmulatorResult<&[u8]> {
        self.protection.verify_address_with_range(range.clone())?;

        if !self
            .protection
            .access_right_with_range(range.clone())
            .is_readable()
        {
            return Err(EmulatorError::AddressNotReadable);
        }

        Ok(&self.inner[range])
    }

    pub(super) fn write_mem(&mut self, address: usize, data: &[u8]) -> EmulatorResult<()> {
        let range = address..address + data.len();
        self.protection.verify_address_with_range(range.clone())?;
        if !self
            .protection
            .access_right_with_range(range.clone())
            .is_writable()
        {
            return Err(EmulatorError::AddressNotWritable);
        }

        self.inner[range].copy_from_slice(data);

        Ok(())
    }

    pub(super) fn set_access_right(
        &mut self,
        range: impl IntoIterator<Item = usize>,
        access_right: AccessRight,
    ) {
        self.protection
            .set_access_right_with_range(range, access_right)
    }
}

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

/// Map each address to its access right.
/// Access right is represented by 2 bits and mapping is done in 2 steps described below.
/// 1. First step is calculating block corresponding to the address. 4 access rights is packed into a single block, thus the block
///    position is calculated by `address / 4`.
/// 2. Second step is extracting the access right from the block. The offset of the access right is calculated by
///    `address % 4 * 2`.
// TODO: Consider better representation.
struct MemoryProtection {
    inner: Vec<u8>,
    memory_size: usize,
}

impl MemoryProtection {
    fn new(memory_size: usize) -> Self {
        let len = if memory_size == 0 {
            0
        } else {
            (memory_size - 1) / 4 + 1
        };
        let inner = vec![0; len];
        Self { inner, memory_size }
    }

    fn set_access_right(&mut self, address: usize, access_right: AccessRight) {
        let block = &mut self.inner[address / 4];
        let offset = address % 4 * 2;
        let mask = !(0b11 << offset);
        *block = (*block & mask) | access_right.as_num() << offset;
    }

    fn access_right(&self, address: usize) -> AccessRight {
        let block = self.inner[address / 4];
        let offset = address % 4 * 2;
        AccessRight::from_num(block >> offset & 0b11)
    }

    fn access_right_with_range(&self, range: impl IntoIterator<Item = usize>) -> AccessRight {
        range
            .into_iter()
            .fold(AccessRight::RW, |acc, i| acc.meet(self.access_right(i)))
    }

    fn set_access_right_with_range(
        &mut self,
        range: impl IntoIterator<Item = usize>,
        access_right: AccessRight,
    ) {
        range
            .into_iter()
            .for_each(|i| self.set_access_right(i, access_right));
    }

    fn verify_address(&self, address: usize) -> EmulatorResult<()> {
        if self.memory_size <= address {
            Err(EmulatorError::InvalidAddress)
        } else {
            Ok(())
        }
    }

    fn verify_address_with_range(
        &self,
        range: impl IntoIterator<Item = usize>,
    ) -> EmulatorResult<()> {
        for i in range {
            self.verify_address(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AccessRight::*;
    use super::*;

    #[test]
    fn test_protection() {
        // [RO, RW, NA, WO, RO];
        let mut protection = MemoryProtection::new(5);
        protection.set_access_right(0, RO);
        protection.set_access_right(1, RW);
        protection.set_access_right(2, NA);
        protection.set_access_right(3, WO);
        protection.set_access_right(4, RO);

        assert_eq!(protection.inner.len(), 2);
        assert_eq!(protection.access_right(0), RO);
        assert_eq!(protection.access_right(1), RW);
        assert_eq!(protection.access_right(2), NA);
        assert_eq!(protection.access_right(3), WO);
        assert_eq!(protection.access_right(4), RO);

        assert_eq!(protection.access_right_with_range(0..2), RO);
        assert_eq!(protection.access_right_with_range(2..4), NA);
        assert_eq!(protection.access_right_with_range(3..5), NA);
    }

    #[test]
    fn test_verify_address() {
        let protection = MemoryProtection::new(5);
        assert!(protection.verify_address(0).is_ok());
        assert!(protection.verify_address(4).is_ok());
        assert!(protection.verify_address(5).is_err());
        assert!(protection.verify_address_with_range(2..5).is_ok());
        assert!(protection.verify_address_with_range(2..6).is_err());
    }

    #[test]
    fn test_read_write_memory() {
        let inner = vec![0, 0, 0, 0];
        let mut mem = Memory::new(inner);
        mem.set_access_right(0..4, RW);

        let data = &[1, 2, 3, 4];
        assert!(mem.write_mem(0, data).is_ok());
        assert_eq!(mem.read_mem(0..4).unwrap(), data);
    }
}
