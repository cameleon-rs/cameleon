use cameleon_macro::byteorder::{ReadBytesExt, LE};
use cameleon_macro::*;
use std::io::Read;

const SBRM_ADDRESS: u64 = 0x1000;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access_right = RO)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access_right = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access_right = RW)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access_right = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {
    assert_eq!(<ABRM as MemoryFragment>::SIZE, 76);

    let raw_entry_local = ABRM::GenCpVersionMajor.into_raw_entry_local();
    assert_eq!(raw_entry_local.offset, 2);
    assert_eq!(raw_entry_local.len, 2);

    let protection = ABRM::memory_protection();
    assert_eq!(protection.access_right_with_range(0..2), AccessRight::RO);
    assert_eq!(
        protection.access_right_with_range(4..4 + 64),
        AccessRight::RW
    );

    let fragment = ABRM::fragment().unwrap();
    let mut cursor = std::io::Cursor::new(&fragment);

    assert_eq!(cursor.read_u16::<LE>().unwrap(), 321);

    cursor.set_position(ABRM::ManufacturerName.into_raw_entry_local().offset as u64);
    let mut buf = vec![0; 9];
    cursor.read(&mut buf).unwrap();
    assert_eq!(
        std::ffi::CStr::from_bytes_with_nul(&buf)
            .unwrap()
            .to_str()
            .unwrap(),
        "Cameleon"
    );

    cursor.set_position(ABRM::SBRMAddress.into_raw_entry_local().offset as u64);
    assert_eq!(cursor.read_u64::<LE>().unwrap(), SBRM_ADDRESS);
}
