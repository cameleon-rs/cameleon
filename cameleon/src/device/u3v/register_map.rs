use std::{convert::TryInto, time::Duration};

use cameleon_device::u3v::{self, register_map::*};

use crate::device::{CompressionType, DeviceError, DeviceResult, GenICamFileType};

use super::control_handle::ControlHandle;

pub struct Abrm<'a> {
    handle: &'a ControlHandle,
    device_capability: DeviceCapability,
}

impl<'a> Abrm<'a> {
    pub fn new(handle: &'a ControlHandle) -> DeviceResult<Self> {
        let (capability_addr, capability_len) = abrm::DEVICE_CAPABILITY;
        let device_capability = read_register(handle, capability_addr, capability_len)?;

        Ok(Self {
            handle,
            device_capability,
        })
    }

    pub fn sbrm(&self) -> DeviceResult<Sbrm> {
        Sbrm::new(self.handle, self.sbrm_address()?)
    }

    pub fn manifest_table(&self) -> DeviceResult<ManifestTable> {
        Ok(ManifestTable::new(
            self.handle,
            self.manifest_table_address()?,
        ))
    }

    pub fn gencp_version(&self) -> DeviceResult<semver::Version> {
        let gencp_version: u32 = self.read_register(abrm::GENCP_VERSION)?;
        let gencp_version_minor = gencp_version & 0xff;
        let gencp_version_major = (gencp_version >> 16) & 0xff;
        Ok(semver::Version::new(
            gencp_version_major as u64,
            gencp_version_minor as u64,
            0,
        ))
    }

    pub fn manufacturer_name(&self) -> DeviceResult<String> {
        self.read_register(abrm::MANUFACTURER_NAME)
    }

    pub fn model_name(&self) -> DeviceResult<String> {
        self.read_register(abrm::MODEL_NAME)
    }

    pub fn family_name(&self) -> DeviceResult<Option<String>> {
        if self.device_capability.is_family_name_supported() {
            self.read_register(abrm::FAMILY_NAME).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn device_version(&self) -> DeviceResult<String> {
        self.read_register(abrm::DEVICE_VERSION)
    }

    pub fn manufacturer_info(&self) -> DeviceResult<String> {
        self.read_register(abrm::MANUFACTURER_INFO)
    }

    pub fn serial_number(&self) -> DeviceResult<String> {
        self.read_register(abrm::SERIAL_NUMBER)
    }

    pub fn user_defined_name(&self) -> DeviceResult<Option<String>> {
        if self.device_capability.is_user_defined_name_supported() {
            self.read_register(abrm::USER_DEFINED_NAME).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn set_user_defined_name(&self, name: &str) -> DeviceResult<()> {
        if !self.device_capability.is_user_defined_name_supported() {
            return Ok(());
        }

        self.write_register(abrm::USER_DEFINED_NAME, name)
    }

    pub fn manifest_table_address(&self) -> DeviceResult<u64> {
        self.read_register(abrm::MANIFEST_TABLE_ADDRESS)
    }

    pub fn sbrm_address(&self) -> DeviceResult<u64> {
        self.read_register(abrm::SBRM_ADDRESS)
    }

    pub fn timestamp(&self) -> DeviceResult<u64> {
        self.read_register(abrm::TIMESTAMP_LATCH)
    }

    pub fn set_timestamp_latch_bit(&self) -> DeviceResult<()> {
        self.write_register(abrm::TIMESTAMP_LATCH, 1u32)
    }

    pub fn timestamp_increment(&self) -> DeviceResult<u64> {
        self.read_register(abrm::TIMESTAMP_INCREMENT)
    }

    pub fn device_software_interface_version(&self) -> DeviceResult<Option<String>> {
        if self
            .device_capability
            .is_device_software_interface_version_supported()
        {
            self.read_register(abrm::DEVICE_SOFTWARE_INTERFACE_VERSION)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn maximum_device_response_time(&self) -> DeviceResult<Duration> {
        self.read_register(abrm::MAXIMUM_DEVICE_RESPONSE_TIME)
    }

    pub fn device_capability(&self) -> DeviceResult<DeviceCapability> {
        Ok(self.device_capability)
    }

    pub fn device_configuration(&self) -> DeviceResult<DeviceConfiguration> {
        self.read_register(abrm::DEVICE_CONFIGURATION)
    }

    pub fn write_device_configuration(&self, config: &DeviceConfiguration) -> DeviceResult<()> {
        self.write_register(abrm::DEVICE_CONFIGURATION, config)
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        read_register(self.handle, register.0, register.1)
    }

    fn write_register(&self, register: (u64, u16), data: impl DumpBytes) -> DeviceResult<()> {
        let (addr, len) = register;
        let mut buf = vec![0; len as usize];
        data.dump_bytes(&mut buf)?;
        self.handle.write_mem(addr, &buf)
    }
}

pub struct Sbrm<'a> {
    handle: &'a ControlHandle,
    sbrm_addr: u64,
    capability: U3VCapablitiy,
}

impl<'a> Sbrm<'a> {
    pub(super) fn new(handle: &'a ControlHandle, sbrm_addr: u64) -> DeviceResult<Self> {
        let (capability_offset, capability_len) = sbrm::U3VCP_CAPABILITY_REGISTER;
        let capability_addr = capability_offset + sbrm_addr;
        let capability = read_register(handle, capability_addr, capability_len)?;

        Ok(Self {
            handle,
            sbrm_addr,
            capability,
        })
    }

    pub fn u3v_version(&self) -> DeviceResult<semver::Version> {
        let u3v_version: u32 = self.read_register(sbrm::U3V_VERSION)?;
        let u3v_version_minor = u3v_version & 0xff;
        let u3v_version_major = (u3v_version >> 16) & 0xff;

        Ok(semver::Version::new(
            u3v_version_major as u64,
            u3v_version_minor as u64,
            0,
        ))
    }

    pub fn maximum_command_transfer_length(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::MAXIMUM_COMMAND_TRANSFER_LENGTH)
    }

    pub fn maximum_acknowledge_trasfer_length(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::MAXIMUM_ACKNOWLEDGE_TRANSFER_LENGTH)
    }

    pub fn number_of_stream_channel(&self) -> DeviceResult<u32> {
        self.read_register(sbrm::NUMBER_OF_STREAM_CHANNELS)
    }

    pub fn sirm_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_sirm_available() {
            self.read_register(sbrm::SIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn sirm_length(&self) -> DeviceResult<Option<u32>> {
        if self.capability.is_sirm_available() {
            self.read_register(sbrm::SIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn eirm_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_eirm_available() {
            self.read_register(sbrm::EIRM_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn eirm_length(&self) -> DeviceResult<Option<u32>> {
        if self.capability.is_eirm_available() {
            self.read_register(sbrm::EIRM_LENGTH).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn iidc2_address(&self) -> DeviceResult<Option<u64>> {
        if self.capability.is_iidc2_available() {
            self.read_register(sbrm::IIDC2_ADDRESS).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn current_speed(&self) -> DeviceResult<u3v::BusSpeed> {
        self.read_register(sbrm::CURRENT_SPEED)
    }

    pub fn u3v_capability(&self) -> DeviceResult<U3VCapablitiy> {
        Ok(self.capability)
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        let addr = offset + self.sbrm_addr;
        read_register(self.handle, addr, len)
    }
}

pub struct ManifestTable<'a> {
    handle: &'a ControlHandle,
    manifest_address: u64,
}

impl<'a> ManifestTable<'a> {
    pub fn new(handle: &'a ControlHandle, manifest_address: u64) -> Self {
        Self {
            handle,
            manifest_address,
        }
    }
    pub fn entries(&self) -> DeviceResult<impl Iterator<Item = ManifestEntry<'a>>> {
        let entry_num: u64 = self.read_register((0, 8))?;
        let first_entry_addr = self.manifest_address + 8;
        let handle = self.handle;

        Ok((0..entry_num)
            .into_iter()
            .map(move |i| ManifestEntry::new(first_entry_addr + i * 64, handle)))
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        read_register(self.handle, offset + self.manifest_address, len)
    }
}

pub struct ManifestEntry<'a> {
    entry_addr: u64,
    handle: &'a ControlHandle,
}

impl<'a> ManifestEntry<'a> {
    pub fn new(entry_addr: u64, handle: &'a ControlHandle) -> Self {
        Self { entry_addr, handle }
    }

    pub fn genicam_file_version(&self) -> DeviceResult<semver::Version> {
        let file_version: u32 = self.read_register(manifest_entry::GENICAM_FILE_VERSION)?;
        let subminor = file_version & 0xff;
        let minor = (file_version >> 16) & 0xff;
        let major = (file_version >> 24) & 0xff;

        Ok(semver::Version::new(
            major as u64,
            minor as u64,
            subminor as u64,
        ))
    }

    /// Register address where GenApi XML file is located.
    pub fn file_address(&self) -> DeviceResult<u64> {
        self.read_register(manifest_entry::REGISTER_ADDRESS)
    }

    /// GenApi XML file size in bytes.
    pub fn file_size(&self) -> DeviceResult<u64> {
        self.read_register(manifest_entry::FILE_SIZE)
    }

    /// GenApi XML file info.
    pub fn file_info(&self) -> DeviceResult<GenICamFileInfo> {
        self.read_register(manifest_entry::FILE_FORMAT_INFO)
    }

    /// SHA1 hash of the file. In case the hash is not available, return None.
    pub fn sha1_hash(&self) -> DeviceResult<Option<[u8; 20]>> {
        // We don't use `self.read_register` here for perf.
        let mut sha1_hash: [u8; 20] = [0; 20];
        let addr = self.entry_addr + manifest_entry::SHA1_HASH.0;
        self.handle.read_mem(addr, &mut sha1_hash)?;

        // All bytes are 0 in case the hash is not available.
        if sha1_hash.iter().all(|byte| *byte == 0) {
            Ok(None)
        } else {
            Ok(Some(sha1_hash))
        }
    }

    fn read_register<T>(&self, register: (u64, u16)) -> DeviceResult<T>
    where
        T: ParseBytes,
    {
        let (offset, len) = register;
        let addr = offset + self.entry_addr;
        read_register(self.handle, addr, len)
    }
}

/// Read and parse register value.
fn read_register<T>(handle: &ControlHandle, addr: u64, len: u16) -> DeviceResult<T>
where
    T: ParseBytes,
{
    let len = len as usize;
    let mut buf = vec![0; len];
    handle.read_mem(addr, &mut buf[..len])?;
    T::parse_bytes(&buf[..len])
}

macro_rules! is_bit_set {
    ($val:expr, $bit:expr) => {
        (($val >> $bit) & 1) == 1
    };
}

macro_rules! set_bit {
    ($val:expr,  $bit:expr) => {
        $val |= (1 << $bit)
    };
}

macro_rules! unset_bit {
    ($val:expr,  $bit:expr) => {
        $val &= !(1 << $bit)
    };
}

#[derive(Clone, Copy)]
pub struct DeviceConfiguration(u64);
impl DeviceConfiguration {
    pub fn is_multi_event_enabled(&self) -> bool {
        is_bit_set!(&self.0, 1)
    }

    pub fn enable_multi_event(&mut self) {
        set_bit!(self.0, 1)
    }

    pub fn disable_multi_event(&mut self) {
        unset_bit!(self.0, 1)
    }
}

#[derive(Clone, Copy)]
pub struct DeviceCapability(u64);

impl DeviceCapability {
    pub fn is_user_defined_name_supported(&self) -> bool {
        is_bit_set!(&self.0, 0)
    }

    pub fn is_family_name_supported(&self) -> bool {
        is_bit_set!(&self.0, 8)
    }

    /// Indicate whether the device supports multiple events in a single event command packet.
    pub fn is_multi_event_supported(&self) -> bool {
        is_bit_set!(&self.0, 12)
    }

    /// Indicate whether the device supports stacked commands (ReadMemStacked and WriteMemStacked).
    pub fn is_stacked_commands_supported(&self) -> bool {
        is_bit_set!(&self.0, 13)
    }

    /// Indicate whether the device supports software interface version is supported.
    pub fn is_device_software_interface_version_supported(&self) -> bool {
        is_bit_set!(&self.0, 14)
    }
}

#[derive(Clone, Copy)]
pub struct U3VCapablitiy(u64);

impl U3VCapablitiy {
    fn is_sirm_available(&self) -> bool {
        is_bit_set!(&self.0, 0)
    }

    fn is_eirm_available(&self) -> bool {
        is_bit_set!(&self.0, 1)
    }

    fn is_iidc2_available(&self) -> bool {
        is_bit_set!(&self.0, 2)
    }
}

pub struct GenICamFileInfo(u32);
impl GenICamFileInfo {
    pub fn file_type(&self) -> DeviceResult<GenICamFileType> {
        let raw = self.0 & 0b111;
        match raw {
            0 => Ok(GenICamFileType::DeviceXml),
            1 => Ok(GenICamFileType::BufferXml),
            _ => Err(DeviceError::InternalError(
                format!("Invalid U3V GenICamFileType value: {}", raw).into(),
            )),
        }
    }

    pub fn compression_type(&self) -> DeviceResult<CompressionType> {
        let raw = (self.0 >> 10) & 0b111111;
        match raw {
            0 => Ok(CompressionType::Uncompressed),
            1 => Ok(CompressionType::Zip),
            _ => Err(DeviceError::InternalError(
                format!("Invalid U3V GenICamFilFormat value: {}", raw).into(),
            )),
        }
    }

    pub fn schema_version(&self) -> semver::Version {
        let major = (self.0 >> 24) & 0xff;
        let minor = (self.0 >> 16) & 0xff;
        semver::Version::new(major as u64, minor as u64, 0)
    }
}

trait ParseBytes: Sized {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self>;
}

impl ParseBytes for DeviceConfiguration {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for DeviceCapability {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u64::parse_bytes(bytes)?))
    }
}

impl ParseBytes for GenICamFileInfo {
    fn parse_bytes(bytes: &[u8]) -> DeviceResult<Self> {
        Ok(Self(u32::parse_bytes(bytes)?))
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
        Ok(Self(u64::parse_bytes(bytes)?))
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

impl<T> DumpBytes for &T
where
    T: DumpBytes,
{
    fn dump_bytes(&self, buf: &mut [u8]) -> DeviceResult<()> {
        (*self).dump_bytes(buf)
    }
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
        self.0.dump_bytes(buf)
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
