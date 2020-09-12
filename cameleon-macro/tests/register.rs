use cameleon_macro::*;

//const SBRM_ADDRESS: u64 = 0x1000;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access_right = RO)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access_right = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access_right = RW)]
    ManufacturerName = "Cameleon",

    #[entry(len = 8, access_right = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {
    let raw_entry_local = ABRM::GenCpVersionMajor.into_raw_entry_local();
    assert_eq!(raw_entry_local.offset, 2);
    assert_eq!(raw_entry_local.len, 2);
    let protection = ABRM::memory_protection();
    assert_eq!(protection.access_right_with_range(0..2), AccessRight::RO);
    assert_eq!(
        protection.access_right_with_range(4..4 + 64),
        AccessRight::RW
    );
}
