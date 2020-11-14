use cameleon_impl::memory::{memory, register_map};

const SBRM_ADDRESS: u64 = 0xffff;
const SIRM_ADDRESS: u64 = (SBRM::base() + SBRM::size()) as u64;

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
    0b1001, 0b0000, 0b1111, 0b0010, 0b0000, 0b0000, 0b0000, 0b0000,
];

/// Offset | Value | Description.
///      0 |     0 | Heartbeat is not used.
///      1 |     0 | MultiEvent is not enabled.
///   2-63 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CONFIGURATION: &[u8] = &[
    0b0000, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000,
];

/// Offset | Value | Description.
///      0 |     1 | SIRM is available.
///      1 |     1 | EIRM is available.
///      2 |     0 | IIDC is NOT available.
///   3-63 |     0 | Reserved. All remained bits are set to 0.
const U3V_CAPABILITY: &[u8] = &[
    0b1100, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000, 0b0000,
];

#[memory]
pub(super) struct Memory {
    abrm: ABRM,
    sbrm: SBRM,
}

#[register_map(base = 0, endianness = LE)]
pub(super) enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 1,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor = 1,

    #[register(len = 64, access = RO, ty = String)]
    ManufacturerName = "cameleon",

    #[register(len = 64, access = RO, ty = String)]
    ModelName = "cameleon model",

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
    ManifestTableAddress, // TODO: Define manifest table address,

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

    #[register(len = 4, access = RO, ty = u32)]
    MaximumCommandTransferLength = 1024,

    #[register(len = 4, access = RO, ty = u32)]
    MaximumAcknowledgeTransferLength = 1024,

    #[register(len = 4, access = RO, ty = u32)]
    NumberOfStrewamChannel = 1,

    #[register(len = 8, access = RO, ty = u64)]
    SirmAddress = SIRM_ADDRESS,

    #[register(len = 4, access = RO, ty = u32)]
    SirmLength, // TODO: Filled after SIRM register map is implemented.

    #[register(len = 8, access = RO, ty = u64)]
    EirmAddress, // TODO: Filled after SIRM register map is implemented.

    #[register(len = 4, access = RO, ty = u32)]
    EirmLength, // TODO: Filled after EIRM register map is implmeneted.

    #[register(len = 8, access = NA, ty = u64)]
    Iidc2Address,

    #[register(len=4, access = RO, ty=u32)]
    CurrentSpeed = 0b1000,
}
