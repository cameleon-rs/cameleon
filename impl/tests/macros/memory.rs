/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::memory::{memory, prelude::*, register_map, AccessRight};

const SBRM_ADDRESS: u64 = 0x1000;
const SIRM_ADDRESS: u64 = 0x2000;
const EIRM_ADDRESS: u64 = 0x3000;
const DEVICE_CAPABILITY: &[u8] = &[
    0b1001, 0b0000, 0b1111, 0b0110, 0b0000, 0b0000, 0b0000, 0b0000,
];

#[memory]
pub struct Memory {
    sbrm: SBRM,
    abrm: ABRM,
}

#[register_map(base = 0, endianness = LE)]
enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon",

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,

    #[register(len = 8, access = RO, ty = Bytes)]
    DeviceCapability = DEVICE_CAPABILITY,

    #[register(len = 4, access = RO, ty = Bytes)]
    ProtocolEndianness = &[0x11, 0x22, 0x33, 0x44],
}

#[register_map(base = SBRM_ADDRESS, endianness = BE)]
enum SBRM {
    #[register(len = 8, access = RO, ty = u64)]
    SIRMAddress = SIRM_ADDRESS,

    #[register(len = 4, access = RO, ty = u32)]
    SIRMLength = 0x20,

    #[register(len = 8, access = RO, ty = u64)]
    EIRMAddress = EIRM_ADDRESS,

    #[register(len = 4, access = RO, ty = u32)]
    EIRMLength = 0x20,

    #[register(len = 1, access = RO, ty = i8)]
    TestI8 = i8::MIN,

    #[register(len = 2, access = RO, ty = i16)]
    TestI16 = i16::MIN,

    #[register(len = 4, access = RO, ty = i32)]
    TestI32 = i32::MIN,

    #[register(len = 8, access = RO, ty = i64)]
    TestI64 = i64::MIN,

    #[register(len = 4, access = RO, ty = f32)]
    TestF32 = 0.291,

    #[register(len = 8, access = RO, ty = f64)]
    TestF64 = 0.27,
}

fn main() {
    let mut memory = Memory::new();

    // Test read.
    let gen_cp_minor = memory.read::<ABRM::GenCpVersionMinor>().unwrap();
    assert_eq!(gen_cp_minor, 321);

    let sbrm_address = memory.read::<ABRM::SBRMAddress>().unwrap();
    assert_eq!(sbrm_address, SBRM_ADDRESS);

    let manufacturer_name = memory.read::<ABRM::ManufacturerName>().unwrap();
    assert_eq!(&manufacturer_name, "Cameleon");

    let sirm_address = memory.read::<SBRM::SIRMAddress>().unwrap();
    assert_eq!(sirm_address, SIRM_ADDRESS);

    let device_capability = memory.read::<ABRM::DeviceCapability>().unwrap();
    assert_eq!(device_capability.as_slice(), DEVICE_CAPABILITY);

    let protocol_endianness = memory.read::<ABRM::ProtocolEndianness>().unwrap();
    assert_eq!(protocol_endianness.as_slice(), &[0x11, 0x22, 0x33, 0x44]);

    assert_eq!(memory.read::<SBRM::TestI8>().unwrap(), i8::MIN);
    assert_eq!(memory.read::<SBRM::TestI16>().unwrap(), i16::MIN);
    assert_eq!(memory.read::<SBRM::TestI32>().unwrap(), i32::MIN);
    assert_eq!(memory.read::<SBRM::TestI64>().unwrap(), i64::MIN);
    assert!((memory.read::<SBRM::TestF32>().unwrap() - 0.291).abs() < f32::EPSILON);
    assert!((memory.read::<SBRM::TestF64>().unwrap() - 0.27).abs() < f64::EPSILON);

    // Test write.
    memory.write::<SBRM::TestI32>(101).unwrap();
    assert_eq!(memory.read::<SBRM::TestI32>().unwrap(), 101);

    memory.write::<SBRM::TestF64>(0.1323).unwrap();
    assert!((memory.read::<SBRM::TestF64>().unwrap() - 0.1323).abs() < f64::EPSILON);

    memory
        .write::<ABRM::ManufacturerName>("New name".into())
        .unwrap();
    let manufacturer_name = memory.read::<ABRM::ManufacturerName>().unwrap();
    assert_eq!(&manufacturer_name, "New name");

    assert_eq!(memory.access_right::<SBRM::EIRMLength>(), AccessRight::RO);
    memory.set_access_right::<SBRM::EIRMLength>(AccessRight::NA);
    assert_eq!(memory.access_right::<SBRM::EIRMLength>(), AccessRight::NA);

    assert!(memory.read_raw(1000..1004).is_err());
}
