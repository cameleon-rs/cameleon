use cameleon_impl::memory::register;

const SBRM_ADDRESS: u64 = 0x1000;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access = RW)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
