use cameleon_impl::register;

const SBRM_ADDRESS: u64 = 0x1000;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access_right = RO)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access_right = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access_right = RW)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access_right = RO)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
