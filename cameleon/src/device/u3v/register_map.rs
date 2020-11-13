use std::{convert::TryInto, time::Duration};

use cameleon_device::u3v::{self, register_map::*};

use crate::device::{DeviceError, DeviceResult};

use super::control_handle::ControlHandle;

pub struct Abrm<'a> {
    abrm: &'a AbrmStaticData,
    handle: &'a ControlHandle,
}

impl<'a> Abrm<'a> {
    pub fn gencp_version(&self) -> semver::Version {
        self.abrm.gencp_version.clone()
    }

    pub fn manufacturer_name(&self) -> &'a str {
        &self.abrm.manufacturer_name
    }

    pub fn model_name(&self) -> &'a str {
        &self.abrm.model_name
    }

    pub fn family_name(&self) -> Option<&'a str> {
        self.abrm.family_name.as_deref()
    }

    pub fn device_version(&self) -> &'a str {
        &self.abrm.device_version
    }

    pub fn manufacturer_info(&self) -> &'a str {
        &self.abrm.manufacturer_info
    }

    pub fn serial_number(&self) -> &'a str {
        &self.abrm.serial_number
    }

    pub fn user_defined_name(&self) -> DeviceResult<Option<String>> {
        if self.is_user_defined_name_supported() {
            let (addr, len) = abrm::USER_DEFINED_NAME;
            read_register(self.handle, &mut vec![], addr, len).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn set_user_defined_name(&self, name: &str) -> DeviceResult<()> {
        if !self.is_user_defined_name_supported() {
            return Ok(());
        }

        let (addr, len) = abrm::USER_DEFINED_NAME;
        let mut buf = vec![0; len as usize];
        name.dump_bytes(&mut buf)?;

        self.handle.write_mem(addr, &buf)
    }

    pub fn manifest_table_address(&self) -> u64 {
        self.abrm.manifest_table_address
    }

    pub fn sbrm_address(&self) -> u64 {
        self.abrm.sbrm_address
    }

    pub fn timestamp(&self) -> DeviceResult<u64> {
        let (addr, len) = abrm::TIMESTAMP_LATCH;
        read_register(&self.handle, &mut vec![], addr, len)
    }

    pub fn set_timestamp_latch_bit(&self) -> DeviceResult<()> {
        let (addr, len) = abrm::TIMESTAMP_LATCH;
        let mut buf = vec![0; len as usize];
        1u32.dump_bytes(&mut buf)?;

        self.handle.write_mem(addr, &buf)
    }

    pub fn timestamp_increment(&self) -> u64 {
        self.abrm.timestamp_increment
    }

    pub fn device_software_interface_version(&self) -> Option<&'a str> {
        self.abrm.device_software_interface_version.as_deref()
    }

    pub fn maximum_device_response_time(&self) -> Duration {
        self.abrm.maximum_device_response_time
    }

    pub fn enable_multi_event(&self) -> DeviceResult<()> {
        let mut config = self.device_configuration()?;
        if !config.is_multi_event_enabled() {
            config.enable_multi_event();
            self.write_device_configuration(&config)
        } else {
            Ok(())
        }
    }

    pub fn disable_multi_event(&self) -> DeviceResult<()> {
        let mut config = self.device_configuration()?;
        if config.is_multi_event_enabled() {
            config.disable_multi_event();
            self.write_device_configuration(&config)
        } else {
            Ok(())
        }
    }

    pub fn is_user_defined_name_supported(&self) -> bool {
        self.abrm.device_capability.is_user_defined_name_supported()
    }

    pub fn is_multi_event_enabled(&self) -> DeviceResult<bool> {
        if self.abrm.device_capability.is_multi_event_supported() {
            let config = self.device_configuration()?;
            Ok(config.is_multi_event_enabled())
        } else {
            Ok(false)
        }
    }

    pub fn is_multi_event_supported(&self) -> bool {
        self.abrm.device_capability.is_multi_event_supported()
    }

    pub fn is_stacked_commands_supported(&self) -> bool {
        self.abrm.device_capability.is_stacked_commands_supported()
    }

    pub(super) fn new(abrm: &'a AbrmStaticData, handle: &'a ControlHandle) -> Self {
        Self { abrm, handle }
    }

    fn device_configuration(&self) -> DeviceResult<DeviceConfiguration> {
        let (addr, len) = abrm::DEVICE_CONFIGURATION;
        read_register(&self.handle, &mut vec![], addr, len)
    }

    fn write_device_configuration(&self, config: &DeviceConfiguration) -> DeviceResult<()> {
        let (addr, len) = abrm::DEVICE_CONFIGURATION;
        let mut buf = vec![0; len as usize];
        config.dump_bytes(&mut buf)?;
        self.handle.write_mem(addr, &buf)
    }
}

pub struct Sbrm<'a> {
    sbrm: &'a SbrmStaticData,
}

impl<'a> Sbrm<'a> {
    pub fn u3v_version(&self) -> &'a semver::Version {
        &self.sbrm.u3v_version
    }

    pub fn maximum_command_transfer_length(&self) -> u32 {
        self.sbrm.maximum_command_transfer_length
    }

    pub fn maximum_acknowledge_trasfer_length(&self) -> u32 {
        self.sbrm.maximum_acknowledge_trasfer_length
    }

    pub fn number_of_stream_channel(&self) -> u32 {
        self.sbrm.number_of_stream_channel
    }

    pub fn sirm_address(&self) -> Option<u64> {
        self.sbrm.sirm_address
    }

    pub fn sirm_length(&self) -> Option<u32> {
        self.sbrm.sirm_length
    }

    pub fn eirm_address(&self) -> Option<u64> {
        self.sbrm.eirm_address
    }

    pub fn eirm_length(&self) -> Option<u32> {
        self.sbrm.eirm_length
    }

    pub fn iidc2_address(&self) -> Option<u64> {
        self.sbrm.iidc2_address
    }

    pub fn current_speed(&self) -> u3v::BusSpeed {
        self.sbrm.current_speed
    }

    pub fn is_sirm_available(&self) -> bool {
        self.sbrm.u3v_capability.is_sirm_available()
    }

    pub fn is_eirm_available(&self) -> bool {
        self.sbrm.u3v_capability.is_eirm_available()
    }

    pub fn is_iidc2_available(&self) -> bool {
        self.sbrm.u3v_capability.is_iidc2_available()
    }

    pub(super) fn new(sbrm: &'a SbrmStaticData) -> Self {
        Self { sbrm }
    }
}

pub(super) struct AbrmStaticData {
    gencp_version: semver::Version,
    manufacturer_name: String,
    model_name: String,
    family_name: Option<String>,
    device_version: String,
    manufacturer_info: String,
    serial_number: String,
    device_capability: DeviceCapability,
    pub(super) maximum_device_response_time: Duration,
    manifest_table_address: u64,
    pub(super) sbrm_address: u64,
    timestamp_increment: u64,
    device_software_interface_version: Option<String>,
}

impl AbrmStaticData {
    pub(super) fn new(handle: &ControlHandle) -> DeviceResult<Self> {
        use abrm::*;

        let mut buf = vec![0; 64];

        macro_rules! read_register {
            ($register_info:ident) => {
                read_register(handle, &mut buf, $register_info.0, $register_info.1)
            };
        }

        let device_capability: DeviceCapability = read_register!(DEVICE_CAPABILITY)?;

        let gencp_version = read_register!(GENCP_VERSION)?;

        let manufacturer_name = read_register!(MANUFACTURER_NAME)?;

        let model_name = read_register!(MODEL_NAME)?;

        let family_name = if device_capability.is_family_name_supported() {
            Some(read_register!(FAMILY_NAME)?)
        } else {
            None
        };

        let device_version = read_register!(DEVICE_VERSION)?;

        let manufacturer_info = read_register!(MANUFACTURER_INFO)?;

        let serial_number = read_register!(SERIAL_NUMBER)?;

        let maximum_device_response_time = read_register!(MAXIMUM_DEVICE_RESPONSE_TIME)?;

        let manifest_table_address = read_register!(MANIFEST_TABLE_ADDRESS)?;

        let sbrm_address = read_register!(SBRM_ADDRESS)?;

        let timestamp_increment = read_register!(TIMESTAMP_INCREMENT)?;

        let device_software_interface_version =
            if device_capability.is_device_software_interface_version_supported() {
                Some(read_register!(DEVICE_SOFTWARE_INTERFACE_VERSION)?)
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
            device_capability,
            maximum_device_response_time,
            manifest_table_address,
            sbrm_address,
            timestamp_increment,
            device_software_interface_version,
        })
    }
}

pub(super) struct SbrmStaticData {
    u3v_version: semver::Version,
    u3v_capability: U3VCapablitiy,
    pub(super) maximum_command_transfer_length: u32,
    pub(super) maximum_acknowledge_trasfer_length: u32,
    number_of_stream_channel: u32,
    pub(super) sirm_address: Option<u64>,
    pub(super) sirm_length: Option<u32>,
    pub(super) eirm_address: Option<u64>,
    pub(super) eirm_length: Option<u32>,
    iidc2_address: Option<u64>,
    current_speed: u3v::BusSpeed,
}

impl SbrmStaticData {
    pub(super) fn new(sbrm_addr: u64, handle: &ControlHandle) -> DeviceResult<Self> {
        use sbrm::*;

        let mut buf = vec![0; 64];
        macro_rules! read_register {
            ($register_info:ident) => {
                read_register(
                    handle,
                    &mut buf,
                    $register_info.0 + sbrm_addr,
                    $register_info.1,
                )
            };
        }

        let u3v_version = read_register!(U3V_VERSION)?;

        let u3v_capability: U3VCapablitiy = read_register!(U3VCP_CAPABILITY_REGISTER)?;

        let maximum_command_transfer_length = read_register!(MAXIMUM_COMMAND_TRANSFER_LENGTH)?;

        let maximum_acknowledge_trasfer_length =
            read_register!(MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH)?;

        let number_of_stream_channel = read_register!(NUMBER_OF_STREAM_CHANNELS)?;

        let (sirm_address, sirm_length) = if u3v_capability.is_sirm_available() {
            (
                Some(read_register!(SIRM_ADDRESS)?),
                Some(read_register!(SIRM_LENGTH)?),
            )
        } else {
            (None, None)
        };

        let (eirm_address, eirm_length) = if u3v_capability.is_eirm_available() {
            (
                Some(read_register!(EIRM_ADDRESS)?),
                Some(read_register!(EIRM_LENGTH)?),
            )
        } else {
            (None, None)
        };

        let iidc2_address = if u3v_capability.is_iidc2_available() {
            Some(read_register!(IIDC2_ADDRESS)?)
        } else {
            None
        };

        let current_speed = read_register!(CURRENT_SPEED)?;

        Ok(Self {
            u3v_version,
            u3v_capability,
            maximum_command_transfer_length,
            maximum_acknowledge_trasfer_length,
            number_of_stream_channel,
            sirm_address,
            sirm_length,
            eirm_address,
            eirm_length,
            iidc2_address,
            current_speed,
        })
    }
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

pub struct DeviceConfiguration([u8; 8]);

impl DeviceConfiguration {
    pub fn is_multi_event_enabled(&self) -> bool {
        is_bit_set(&self.0, 1)
    }

    pub fn enable_multi_event(&mut self) {
        set_bit(&mut self.0, 1)
    }

    pub fn disable_multi_event(&mut self) {
        unset_bit(&mut self.0, 1)
    }
}

struct DeviceCapability([u8; 8]);

impl DeviceCapability {
    fn is_user_defined_name_supported(&self) -> bool {
        is_bit_set(&self.0, 0)
    }

    fn is_family_name_supported(&self) -> bool {
        is_bit_set(&self.0, 8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    fn is_multi_event_supported(&self) -> bool {
        is_bit_set(&self.0, 12)
    }

    /// Indicate whether the device supports stacked commands (ReadMemStacked and WriteMemStacked).
    fn is_stacked_commands_supported(&self) -> bool {
        is_bit_set(&self.0, 13)
    }

    /// Indicate whether the device supports software interface version is supported.
    fn is_device_software_interface_version_supported(&self) -> bool {
        is_bit_set(&self.0, 14)
    }
}

struct U3VCapablitiy([u8; 8]);
impl U3VCapablitiy {
    fn is_sirm_available(&self) -> bool {
        is_bit_set(&self.0, 0)
    }

    fn is_eirm_available(&self) -> bool {
        is_bit_set(&self.0, 1)
    }

    fn is_iidc2_available(&self) -> bool {
        is_bit_set(&self.0, 2)
    }
}

fn is_bit_set(bytes: &[u8], offset: usize) -> bool {
    debug_assert!(offset < bytes.len() * 8);
    let idx = offset / 8;
    let rem = offset % 8;
    (bytes[idx] >> rem) & 1 == 1
}

fn set_bit(bytes: &mut [u8], offset: usize) {
    debug_assert!(offset < bytes.len() * 8);

    let idx = offset / 8;
    let rem = offset % 8;
    bytes[idx] |= 1 << rem;
}

fn unset_bit(bytes: &mut [u8], offset: usize) {
    debug_assert!(offset < bytes.len() * 8);

    let idx = offset / 8;
    let rem = offset % 8;
    bytes[idx] &= !(1 << rem);
}

trait ParseBytes: Sized {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self>;
}

impl ParseBytes for semver::Version {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        let minor = u16::parse_bytes(&bytes[0..2])?;
        let major = u16::parse_bytes(&bytes[2..])?;
        Ok(semver::Version::new(major as u64, minor as u64, 0))
    }
}

impl ParseBytes for DeviceConfiguration {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(bytes.try_into().unwrap()))
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(bytes.try_into().unwrap()))
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
        let raw = u32::parse_bytes(bytes)?;
        Ok(Duration::from_millis(raw as u64))
    }
}

impl ParseBytes for U3VCapablitiy {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(bytes.try_into().unwrap()))
    }
}

impl ParseBytes for u3v::BusSpeed {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        use u3v::BusSpeed::*;

        let raw = u32::parse_bytes(bytes)?;
        let speed = match raw {
            0b1 => LowSpeed,
            0b10 => FullSpeed,
            0b100 => HighSpeed,
            0b1000 => SuperSpeed,
            0b10000 => SuperSpeedPlus,
            other => {
                return Err(DeviceError::InternalError(
                    format!("invalid bus speed defined:  {:#b}", other).into(),
                ))
            }
        };

        Ok(speed)
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

trait DumpBytes {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()>;
}

impl DumpBytes for &str {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        if !self.is_ascii() {
            return Err(DeviceError::InvalidData(
                "string encoding must be ascii".into(),
            ));
        }

        let data_len = self.len();
        if data_len > buf.len() {
            return Err(DeviceError::InvalidData("too large string".into()));
        }

        buf[..data_len].copy_from_slice(self.as_bytes());
        // Zero terminate if data is shorter than buffer length.
        if data_len < buf.len() {
            buf[data_len] = 0;
        }

        Ok(())
    }
}

impl DumpBytes for DeviceConfiguration {
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        debug_assert_eq!(self.0.len(), buf.len());

        buf.copy_from_slice(&self.0);
        Ok(())
    }
}

macro_rules! impl_dump_bytes_for_numeric {
    ($ty:ty) => {
        impl DumpBytes for $ty {
            fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
                let data = self.to_le_bytes();
                debug_assert_eq!(data.len(), buf.len());

                buf.copy_from_slice(&data);
                Ok(())
            }
        }
    };
}

impl_dump_bytes_for_numeric!(u8);
impl_dump_bytes_for_numeric!(u16);
impl_dump_bytes_for_numeric!(u32);
impl_dump_bytes_for_numeric!(u64);
impl_dump_bytes_for_numeric!(i8);
impl_dump_bytes_for_numeric!(i16);
impl_dump_bytes_for_numeric!(i32);
impl_dump_bytes_for_numeric!(i64);
