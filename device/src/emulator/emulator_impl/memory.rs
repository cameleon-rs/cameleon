/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::memory::{memory, register_map, Register};

use super::genapi;

const ABRM_ADDRESS: usize = 0;
const SBRM_ADDRESS: usize = 0xffff;
const SIRM_ADDRESS: usize = SBRM::base() + SBRM::size();
const MANIFEST_TABLE_ADDRESS: usize = SIRM::base() + SIRM::size();
pub(super) const GENAPI_XML_ADDRESS: usize = ManifestTable::base() + ManifestTable::size();
const GENAPI_XML_LENGTH: usize = genapi::GENAPI_XML.len();

/// Offset | Value | Description.
///      0 |     1 | User Defined Name is supported.
///      1 |     0 | Access Privilege and Heartbeat are NOT supported.
///      2 |     0 | Message Channel is NOT supported.
///      3 |     1 | Timestampl is supported.
///    4-7 |  0000 | String Encoding (Ascii).
///      8 |     1 | Family Name is supported.
///      9 |     1 | SBRM is supported.
///     10 |     1 | Endianness Register is supported.
///     11 |     1 | Written Length Field is supported.
///     12 |     0 | Multi Event is currently NOT supported.
///     13 |     0 | Stacked Commands is NOT supported.
///     14 |     1 | Device Software Interface Version is supported.
///  15-63 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CAPABILITY: &[u8] = &[
    0b0000_1001,
    0b0100_1111,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
];

/// Offset | Value | Description.
///      0 |     1 | SIRM is available.
///      1 |     1 | EIRM is available.
///      2 |     0 | IIDC is NOT available.
///   3-63 |     0 | Reserved. All remained bits are set to 0.
const U3V_CAPABILITY: &[u8] = &[
    0b0000_0011,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
];

/// Offset | Value | Description.
///      0 |     0 | Heartbeat is not used.
///      1 |     0 | Multievent is not enabled.
///   2-63 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CONFIGURATION: &[u8] = &[
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,
];

#[memory]
pub(super) struct Memory {
    abrm: ABRM,
    sbrm: SBRM,
    sirm: SIRM,
    manifest_table: ManifestTable,
    genapi_xml: GenApiXml,
}

#[register_map(base = ABRM_ADDRESS, endianness = LE)]
pub(super) enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 1,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor = 1,

    #[register(len = 64, access = RO, ty = String)]
    ManufacturerName = genapi::VENDOR_NAME,

    #[register(len = 64, access = RO, ty = String)]
    ModelName = genapi::MODEL_NAME,

    #[register(len = 64, access = RO, ty = String)]
    FamilyName = "cameleon family",

    #[register(len = 64, access = RO, ty = String)]
    DeviceVersion = "none",

    #[register(len = 64, access = RO, ty = String)]
    ManufacturerInfo = "none",

    #[register(len = 64, access = RO, ty = String)]
    SerialNumber,

    #[register(len = 64, access = RW, ty = String)]
    UserDefinedName,

    #[register(len = 8, access = RO, ty = Bytes)]
    DeviceCapability = DEVICE_CAPABILITY,

    #[register(len = 4, access = RO, ty = u32)]
    MaximumDeviceResponseTime = 500, // 500 ms.

    #[register(len = 8, access = RO, ty = u64)]
    ManifestTableAddress = MANIFEST_TABLE_ADDRESS as u64,

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,

    #[register(len = 8, access = RW, ty = Bytes)]
    DeviceConfiguration = DEVICE_CONFIGURATION,

    #[register(len = 4, access = NA, ty = u32)]
    HeartbeatTimeout,

    #[register(len = 4, access = NA, ty = u32)]
    MessageChannelId,

    #[register(len = 8, access = RO, ty = u64)]
    Timestamp,

    #[register(len = 4, access = WO, ty = u32)]
    TimestampLatch,

    #[register(len = 8, access = RO, ty = u64)]
    TimestampIncrement = 1000, // Dummy value indicating device clock runs at 1MHZ.

    #[register(len = 4, access = NA, ty = Bytes)]
    AccessPrivilege,

    #[register(len = 4, access = RO, ty = Bytes)]
    ProtocolEndianness = &[0xFF, 0xFF, 0xFF, 0xFF], // Little endian.

    #[register(len = 4, access = NA, ty = Bytes)]
    ImplementationEndianness,

    #[register(len = 64, access = RO, ty = String)]
    DeviceSoftwareInterfaceVersion = "1.0.0",
}

#[register_map(base = SBRM_ADDRESS, endianness = LE)]
pub(super) enum SBRM {
    #[register(len = 2, access = RO, ty = u16)]
    U3VVersionMinor = 0,

    #[register(len = 2, access = RO, ty = u16)]
    U3VVersionMajor = 1,

    #[register(len = 8, access = RO, ty = Bytes)]
    U3VCapability = U3V_CAPABILITY,

    #[register(len = 8, access = RW, ty = Bytes)]
    U3VConfiguration, // Not used.

    #[register(len = 4, access = RO, ty = u32)]
    MaximumCommandTransferLength = 1024,

    #[register(len = 4, access = RO, ty = u32)]
    MaximumAcknowledgeTransferLength = 1024,

    #[register(len = 4, access = RO, ty = u32)]
    NumberOfStreamChannel = 1,

    #[register(len = 8, access = RO, ty = u64)]
    SirmAddress = SIRM_ADDRESS,

    #[register(len = 4, access = RO, ty = u32)]
    SirmLength = SIRM::size() as u32,

    #[register(len = 8, access = RO, ty = u64)]
    EirmAddress, // TODO: Filled after SIRM register map is implemented.

    #[register(len = 4, access = RO, ty = u32)]
    EirmLength, // TODO: Filled after EIRM register map is implmeneted.

    #[register(len = 8, access = NA, ty = u64)]
    Iidc2Address,

    #[register(len=4, access = RO, ty=u32)]
    CurrentSpeed = 0b1000,
}

pub(super) const SIRM_ALIGNMENT: u8 = 4;
/// Exponent of alignment is in Upper 8bits of SI Info.
/// TODO: Use `{integer}::log2` when it's stabilized. See <https://github.com/rust-lang/rust/issues/70887>.
const SI_INFO: u32 = 2 << 24;

#[register_map(base = SIRM_ADDRESS, endianness = LE)]
pub(super) enum SIRM {
    #[register(len = 4, access = RO, ty = u32)]
    Info = SI_INFO,

    #[register(len = 4, access = RW, ty = u32)]
    Control = 0,

    #[register(len = 8, access = RO, ty = u64)]
    RequiredPayloadSize = 15_151_104,

    #[register(len = 4, access = RO, ty = u32)]
    RequiredLeaderSize = 1024,

    #[register(len = 4, access = RO, ty = u32)]
    RequiredTrailerSize = 1024,

    #[register(len = 4, access = RW, ty = u32)]
    MaximumLeaderSize = 0,

    #[register(len = 4, access = RW, ty = u32)]
    PayloadTransferSize = 0,

    #[register(len = 4, access = RW, ty = u32)]
    PayloadTransferCount = 0,

    #[register(len = 4, access = RW, ty = u32)]
    PayloadFinalTransferSize1 = 0,

    #[register(len = 4, access = RW, ty = u32)]
    PayloadFinalTransferSize2 = 0,

    #[register(len = 4, access = RW, ty = u32)]
    MaximumTrailerSize = 0,
}

const MANIFEST_ENTRY0_BF_OFFSET: usize = (ManifestTable::GenICamFileVersionMajor::ADDRESS
    + ManifestTable::GenICamFileVersionMajor::LENGTH)
    - MANIFEST_TABLE_ADDRESS;

#[register_map(base = MANIFEST_TABLE_ADDRESS, endianness = LE)]
pub(super) enum ManifestTable {
    #[register(len = 8, access = RO, ty = u64)]
    EntryCount = 1,

    // Manifest Entry 0 start.
    #[register(len = 2, access = RO, ty = u16)]
    GenICamFileVersionSubMinor = genapi::XML_SUBMINOR_VERSION as u16,

    #[register(len = 1, access = RO, ty = u8)]
    GenICamFileVersionMinor = genapi::XML_MINOR_VERSION as u8,

    #[register(len = 1, access = RO, ty = u8)]
    GenICamFileVersionMajor = genapi::XML_MAJOR_VERSION as u8,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 0, MSB = 2>, offset = MANIFEST_ENTRY0_BF_OFFSET)]
    FileType = 0b000, // DeviceXML.

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 10, MSB = 15>, offset = MANIFEST_ENTRY0_BF_OFFSET)]
    FileFormat = 0b00000, // Uncompressed.

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 16, MSB = 23>, offset = MANIFEST_ENTRY0_BF_OFFSET)]
    SchemaVersionMinor = genapi::SCHEME_MINOR_VERSION as u32,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 24, MSB = 31>, offset = MANIFEST_ENTRY0_BF_OFFSET)]
    SchemaVersionMajor = genapi::SCHEME_MAJOR_VERSION as u32,

    #[register(len = 8, access = RO, ty = u64)]
    RegisterAddress = GENAPI_XML_ADDRESS as u64,

    #[register(len = 8, access = RO, ty = u64)]
    FileSize = GENAPI_XML_LENGTH as u64,

    #[register(len = 20, access = RO, ty = Bytes)]
    Sha1Hash, // Hash is not available.

    #[register(len = 20, access = NA, ty = Bytes)]
    _Reserved,
    // Manifest Entry 0 end.
}

#[register_map(base = GENAPI_XML_ADDRESS, endianness = LE)]
pub(super) enum GenApiXml {
    #[register(len = GENAPI_XML_LENGTH, access = RO, ty = String)]
    Xml = genapi::GENAPI_XML,
}
