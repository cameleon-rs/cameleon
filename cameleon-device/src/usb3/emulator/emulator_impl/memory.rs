use cameleon_impl::memory::{memory, register};

const SBRM_ADDRESS: u64 = 0xffff;

// TODO: Multievent support.
/// Offset | Value | Description.
///      0 |     1 | User Defined Name is supported.
///      1 |     0 | Access Privilege and Heartbeat are NOT supported.
///      2 |     0 | Message Channel is NOT supported.
///      3 |     1 | Timestampl is supported.
///    7-4 |  0000 | String Encoding (Ascii).
///      8 |     1 | Family Name is supported.
///      9 |     1 | SBRM is supported.
///     10 |     1 | Endianness Register is supported.
///     11 |     1 | Written Length Field is supported.
///     12 |     0 | Multi Event is currentrly NOT supported.
///     13 |     1 | Stacked Commands is supported.
///     14 |     1 | Device Software Interface Version is supported.
///  63-15 |     0 | Reserved. All remained bits are set to 0.
const DEVICE_CAPABILITY: &[u8] = &[
    0b1001, 0b0000, 0b1111, 0b0110, 0b0000, 0b0000, 0b0000, 0b0000,
];

#[memory]
pub(super) struct Memory {
    abrm: ABRM,
}

#[register(base = 0, endianness = LE)]
pub(super) enum ABRM {
    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor = 1,

    #[entry(len = 64, access = RO, ty = String)]
    ManufacturerName = "cameleon",

    #[entry(len = 64, access = RO, ty = String)]
    ModelName = "cameleon model",

    #[entry(len = 64, access = RO, ty = String)]
    FamilyName = "cameleon family",

    #[entry(len = 64, access = RO, ty = String)]
    DeviceVersion = "none",

    #[entry(len = 64, access = RO, ty = String)]
    ManufacturerInfo = "none",

    #[entry(len = 64, access = RO, ty = String)]
    SerialNumber,

    #[entry(len = 64, access = RW, ty = String)]
    UserDefinedName,

    #[entry(len = 8, access = RO, ty = Bytes)]
    DeviceCapability = DEVICE_CAPABILITY,

    #[entry(len = 4, access = RO, ty = u32)]
    MaximumDeviceResponseTime = 100,

    #[entry(len = 8, access = RO, ty = u64)]
    ManifestTableAddress, // TODO: Define manifest table address,

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,

    #[entry(len = 8, access = RO, ty = u64)]
    DeviceConfiguration,

    #[entry(len = 4, access = NA, ty = u32)]
    HeartbeatTimeout,

    #[entry(len = 4, access = NA, ty = u32)]
    MessageChannelId,

    #[entry(len = 8, access = RO, ty = u64)]
    Timestamp,

    #[entry(len = 4, access = WO, ty = u32)]
    TimestampLatch,

    #[entry(len = 8, access = RO, ty = u64)]
    TimestampIncrement = 1000, // Dummy value indicating device clock runs at 1MHZ.

    #[entry(len = 4, access = NA, ty = Bytes)]
    AccessPrivilege,

    #[entry(len = 4, access = RO, ty = Bytes)]
    ProtocolEndianness = &[0xFF, 0xFF, 0xFF, 0xFF], // Little endian.

    #[entry(len = 4, access = NA, ty = Bytes)]
    ImplementationEndianness,

    #[entry(len = 64, access = RO, ty = String)]
    DeviceSoftwareInterfaceVersion = "1.0.0",
}
