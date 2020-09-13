use cameleon_macro::{memory, register};

const SBRM_ADDRESS: u64 = 0x1000;
const SIRM_ADDRESS: u64 = 0x2000;
const EIRM_ADDRESS: u64 = 0x3000;

#[memory]
pub struct Memory {
    #[offset(0)]
    abrm: ABRM,

    #[offset(SBRM_ADDRESS)]
    sbrm: SBRM,
}

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

#[register(endianess = LE)]
pub enum SBRM {
    #[entry(len = 8, access_right = RO, ty = u64)]
    SIRMAddress = SIRM_ADDRESS,

    #[entry(len = 4, access_right = RO)]
    SIRMLength = 0x20,

    #[entry(len = 4, access_right = RO, ty = u64)]
    EIRMAddress = EIRM_ADDRESS,

    #[entry(len = 4, access_right = RO)]
    EirmLength = 0x20,
}

fn main() {}
