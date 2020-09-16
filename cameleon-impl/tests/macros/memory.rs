use cameleon_impl::memory::{memory, prelude::*, register, AccessRight};

const SBRM_ADDRESS: u64 = 0x1000;
const SIRM_ADDRESS: u64 = 0x2000;
const EIRM_ADDRESS: u64 = 0x3000;

#[memory]
pub struct Memory {
    abrm: ABRM,

    sbrm: SBRM,
}

#[register(base = 0, endianess = LE)]
enum ABRM {
    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[entry(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

#[register(base = SBRM_ADDRESS, endianess = BE)]
enum SBRM {
    #[entry(len = 8, access = RO, ty = u64)]
    SIRMAddress = SIRM_ADDRESS,

    #[entry(len = 4, access = RO, ty = u32)]
    SIRMLength = 0x20,

    #[entry(len = 8, access = RO, ty = u64)]
    EIRMAddress = EIRM_ADDRESS,

    #[entry(len = 4, access = RO, ty = u32)]
    EIRMLength = 0x20,
}

fn main() {
    let mut memory = Memory::new();

    // Test read_entry.
    let gen_cp_minor = memory.read_entry::<ABRM::GenCpVersionMinor>().unwrap();
    assert_eq!(gen_cp_minor, 321);

    let sbrm_address = memory.read_entry::<ABRM::SBRMAddress>().unwrap();
    assert_eq!(sbrm_address, SBRM_ADDRESS);

    let manufacturer_name = memory.read_entry::<ABRM::ManufacturerName>().unwrap();
    assert_eq!(from_zero_terminated_string(&manufacturer_name), "Cameleon");

    let sirm_address = memory.read_entry::<SBRM::SIRMAddress>().unwrap();
    assert_eq!(sirm_address, SIRM_ADDRESS);

    // Test write entry.
    memory
        .write_entry::<ABRM::ManufacturerName>("New name\0".into())
        .unwrap();
    let manufacturer_name = memory.read_entry::<ABRM::ManufacturerName>().unwrap();
    assert_eq!(from_zero_terminated_string(&manufacturer_name), "New name");

    assert_eq!(memory.access_right::<SBRM::EIRMLength>(), AccessRight::RO);
    memory.set_access_right::<SBRM::EIRMLength>(AccessRight::NA);
    assert_eq!(memory.access_right::<SBRM::EIRMLength>(), AccessRight::NA);

    assert!(memory.read(1000..1004).is_err());
}

fn from_zero_terminated_string(s: &str) -> &str {
    let string_len = s.as_bytes().iter().position(|c| *c == 0).unwrap();
    &s[0..string_len]
}
