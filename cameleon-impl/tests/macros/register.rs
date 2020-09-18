use cameleon_impl::memory::*;

const SBRM_ADDRESS: u64 = 0x1000;

#[register(base = 0, endianness = LE)]
pub enum ABRM {
    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[entry(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

#[register(base = SBRM_ADDRESS, endianness = LE)]
pub enum SBRM {
    #[entry(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",
}

fn main() {
    assert_eq!(ABRM::SIZE, 76);

    let raw_entry = ABRM::GenCpVersionMajor::raw_entry();
    assert_eq!(raw_entry.offset, 2);
    assert_eq!(raw_entry.len, 2);

    let protection = ABRM::memory_protection();
    assert_eq!(protection.access_right_with_range(0..2), AccessRight::RO);
    assert_eq!(
        protection.access_right_with_range(4..4 + 64),
        AccessRight::RW
    );

    let raw_entry = SBRM::ManufacturerName::raw_entry();
    assert_eq!(raw_entry.offset, SBRM_ADDRESS as usize);
    assert_eq!(raw_entry.len, 64);
}
