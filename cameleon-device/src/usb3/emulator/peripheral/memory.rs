use std::{borrow::Cow, collections::HashMap, io::Write, ops::Range};

use byteorder::{WriteBytesExt, LE};
use semver::Version;
use thiserror::Error;

use crate::usb3::register_map::*;

use super::device_builder::{BuilderError, BuilderResult};

const SBRM_ADDRESS: u64 = 0xffff;

// TODO: Multievent support.
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
///     12 |     0 | Multi Event is currentrly NOT supported.
///     13 |     1 | Stacked Commands is supported.
///     14 |     1 | Device Software Interface Version is supported.
///  63-15 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CAPABILITY: u64 = 0b111011100001001;

#[derive(Debug, Error)]
pub(super) enum MemoryError {
    #[error("attempt to read unreadable address")]
    AddressNotReadable,

    #[error("attempt to write to unwritable address")]
    AddressNotWritable,

    #[error("attempt to access not existed memory location")]
    InvalidAddress,
}

pub(super) type MemoryResult<T> = std::result::Result<T, MemoryError>;

pub(super) struct Memory {
    inner: Vec<u8>,
    protection: MemoryProtection,
    chain_builder: EventChainBuilder,
}

impl Memory {
    // TODO: Add SBRM, EIRM, SIRM, GenXML.
    pub(super) fn new(abrm: ABRM) -> Self {
        let memory_len = *[abrm.last_address()].iter().max().unwrap();
        let inner = vec![0; memory_len];
        let protection = MemoryProtection::new(memory_len);
        let mut memory = Memory {
            protection,
            inner,
            chain_builder: EventChainBuilder::new(),
        };

        abrm.flush(&mut memory);
        memory
    }

    pub(super) fn read_mem(&self, range: Range<usize>) -> MemoryResult<&[u8]> {
        self.protection.verify_address_with_range(range.clone())?;

        if !self
            .protection
            .access_right_with_range(range.clone())
            .is_readable()
        {
            return Err(MemoryError::AddressNotReadable);
        }

        Ok(&self.inner[range])
    }

    pub(super) fn write_mem(
        &mut self,
        address: usize,
        data: &[u8],
    ) -> MemoryResult<Option<EventChain>> {
        let range = address..address + data.len();
        self.protection.verify_address_with_range(range.clone())?;
        if !self
            .protection
            .access_right_with_range(range.clone())
            .is_writable()
        {
            return Err(MemoryError::AddressNotWritable);
        }

        self.inner[range].copy_from_slice(data);

        Ok(self
            .chain_builder
            .build_chain(address as u64, data.len() as u16)
            .cloned())
    }

    pub(super) fn write_mem_u8_unchecked(&mut self, address: usize, data: u8) {
        self.inner[address] = data;
    }

    pub(super) fn write_mem_u16_unchecked(&mut self, address: usize, data: u16) {
        let range = address..address + 2;
        (&mut self.inner[range]).write_u16::<LE>(data).unwrap();
    }

    pub(super) fn write_mem_u32_unchecked(&mut self, address: usize, data: u32) {
        let range = address..address + 4;
        (&mut self.inner[range]).write_u32::<LE>(data).unwrap();
    }

    pub(super) fn write_mem_u64_unchecked(&mut self, address: usize, data: u64) {
        let range = address..address + 8;
        (&mut self.inner[range]).write_u64::<LE>(data).unwrap();
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

/// Write requests from user may cause "event chain".
/// e.g. write request to TimestampLatch entry causes another write request to Timestamp entry.
/// Or write request to Si control entry causes stream enable event.
///
/// `EventChainBuilder` manage these write requests by observing write requests.
pub(super) struct EventChainBuilder {
    chain_map: HashMap<(u64, u16), EventChain>,
}

impl EventChainBuilder {
    fn new() -> Self {
        Self {
            chain_map: HashMap::new(),
        }
    }

    fn register(&mut self, address: u64, len: u16, chain: EventChain) {
        let must_none = self.chain_map.insert((address, len), chain);
        debug_assert!(must_none.is_none());
    }

    fn build_chain(&self, address: u64, len: u16) -> Option<&EventChain> {
        self.chain_map.get(&(address, len))
    }
}

#[derive(Debug, Clone)]
pub(super) struct EventChain {
    chain: Vec<EventData>,
}

impl EventChain {
    pub(super) fn chain(&self) -> &[EventData] {
        &self.chain
    }

    fn single_event(event_data: EventData) -> Self {
        Self {
            chain: vec![event_data],
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum EventData {
    WriteTimestamp { addr: usize },
}

#[derive(Debug, Clone)]
pub(super) struct ABRM {
    inner: HashMap<(u64, u16, AccessRight), RegisterEntryData>,
    last_address: usize,
}

macro_rules! string_setter {
    ($fn_name:ident, $entry:ident) => {
        pub(super) fn $fn_name(&mut self, name: &str) -> BuilderResult<()> {
            use abrm::*;
            verify_str(name)?;
            *self.inner.get_mut(&$entry).unwrap() = RegisterEntryData::Str(name.to_owned().into());
            Ok(())
        }
    };
}

impl ABRM {
    string_setter!(set_model_name, MODEL_NAME);
    string_setter!(set_family_name, FAMILY_NAME);
    string_setter!(set_serial_number, SERIAL_NUMBER);
    string_setter!(set_user_defined_name, USER_DEFINED_NAME);

    fn flush(&self, memory: &mut Memory) {
        let mem_inner = &mut memory.inner;
        let protection = &mut memory.protection;
        for ((addr, len, right), data) in self.inner.iter() {
            let range = *addr as usize..*addr as usize + *len as usize;

            // Flush entry data to memory.
            data.write(&mut mem_inner[range.clone()]);

            // Set access right to memory protection.
            protection.set_access_right_with_range(range, *right);
        }

        // Register event chains.
        let chain_builder = &mut memory.chain_builder;

        // Register event associated with TimestampLatch ently.
        let timestamp_latch = abrm::TIMESTAMP_LATCH;
        let (addr, len) = (timestamp_latch.0, timestamp_latch.1);
        let chain = EventChain::single_event(EventData::WriteTimestamp {
            addr: abrm::TIMESTAMP.0 as usize,
        });
        chain_builder.register(addr, len, chain);
    }

    fn last_address(&self) -> usize {
        self.last_address
    }

    pub(super) fn version_from(&self, entry: (u64, u16, AccessRight)) -> Version {
        match &self.inner[&entry] {
            RegisterEntryData::Ver(ver) => ver.clone(),
            _ => panic!("That entry doesn't contain label"),
        }
    }

    pub(super) fn string_from(&self, entry: (u64, u16, AccessRight)) -> String {
        match &self.inner[&entry] {
            RegisterEntryData::Str(s) => s.to_string(),
            _ => panic!("That entry doesn't contain string"),
        }
    }
}

impl Default for ABRM {
    fn default() -> Self {
        use rand::seq::SliceRandom;
        use RegisterEntryData::*;

        // Default serial number is 8 length digit picked at random.
        let mut rang = rand::thread_rng();
        let serial_base = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let serial_number: String = (0..8)
            .map(|_| serial_base.choose(&mut rang).unwrap())
            .collect();

        let raw_abrm = [
            (abrm::GENCP_VERSION, Ver(Version::new(1, 3, 0))),
            (abrm::MANUFACTURER_NAME, Str("cameleon".into())),
            (abrm::MODEL_NAME, Str("cameleon model".into())),
            (abrm::FAMILY_NAME, Str("cameleon family".into())),
            (abrm::DEVICE_VERSION, Str("none".into())),
            (abrm::MANUFACTURER_INFO, Str("".into())),
            (abrm::SERIAL_NUMBER, Str(serial_number.into())),
            (abrm::USER_DEFINED_NAME, Str("none".into())),
            (abrm::DEVICE_CAPABILITY, U64(DEVICE_CAPABILITY)),
            (abrm::MAXIMUM_DEVICE_RESPONSE_TIME, U32(100)),
            (abrm::MANIFEST_TABLE_ADDRESS, U64(0xFFFFFFFFFFFFFFFF)), // TODO: Define manifest table address.
            (abrm::SBRM_ADDRESS, U64(SBRM_ADDRESS)),
            (abrm::DEVICE_CONFIGURATION, U64(0)),
            (abrm::HEARTBEAT_TIMEOUT, U32(0)),
            (abrm::MESSAGE_CHANNEL_ID, U32(0)),
            (abrm::TIMESTAMP, U64(0)),
            (abrm::TIMESTAMP_LATCH, U32(0)),
            (abrm::TIMESTAMP_INCREMENT, U64(1000)), // Dummy value indicating device clock runs at 1MHz.
            (abrm::ACCESS_PRIVILEGE, U32(0)),
            (abrm::PROTOCOL_ENDIANESS, U32(0xFFFFFFFF)), // Little endian.
            (abrm::IMPLEMENTATION_ENDIANESS, U32(0xFFFFFFFF)), // Little endian.
            (abrm::DEVICE_SOFTWARE_INTERFACE_VERSION, Str("1.0.0".into())),
        ];

        let (last_addr, last_len, _) = raw_abrm.last().unwrap().0;
        let last_address = last_addr as usize + last_len as usize;
        Self {
            inner: raw_abrm.iter().cloned().collect(),
            last_address,
        }
    }
}

fn verify_str(s: &str) -> BuilderResult<()> {
    const STRING_LENGTH_LIMIT: usize = 64;

    if !s.is_ascii() {
        return Err(BuilderError::InvalidString("string format is not ascii"));
    }

    // String in register must be 0 terminated, so need to subtract 1 from STRING_LENGTH_LIMIT.
    if s.as_bytes().len() > STRING_LENGTH_LIMIT - 1 {
        return Err(BuilderError::InvalidString("string is too long."));
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum RegisterEntryData {
    U16(u16),
    U32(u32),
    U64(u64),
    Ver(Version),
    Str(Cow<'static, str>),
}

impl RegisterEntryData {
    fn write(&self, mut wtr: impl Write) {
        match self {
            Self::U16(data) => wtr.write_u16::<LE>(*data).unwrap(),
            Self::U32(data) => wtr.write_u32::<LE>(*data).unwrap(),
            Self::U64(data) => wtr.write_u64::<LE>(*data).unwrap(),
            Self::Ver(data) => {
                wtr.write_u16::<LE>(data.minor as u16).unwrap();
                wtr.write_u16::<LE>(data.major as u16).unwrap();
            }
            Self::Str(data) => {
                debug_assert!(verify_str(data).is_ok());
                wtr.write_all(data.as_bytes()).unwrap();
                wtr.write_u8(0).unwrap() // 0 terminate.
            }
        }
    }
}

/// Map each address to its access right.
/// Access right is represented by 2 bits and mapping is done in 2 steps described below.
/// 1. First step is calculating block corresponding to the address. Four access rights is packed into a single block, thus the block
///    position is calculated by `address / 4`.
/// 2. Second step is extracting the access right from the block. The offset of the access right is calculated by
///    `address % 4 * 2`.
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

    fn verify_address(&self, address: usize) -> MemoryResult<()> {
        if self.memory_size <= address {
            Err(MemoryError::InvalidAddress)
        } else {
            Ok(())
        }
    }

    fn verify_address_with_range(
        &self,
        range: impl IntoIterator<Item = usize>,
    ) -> MemoryResult<()> {
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
}
