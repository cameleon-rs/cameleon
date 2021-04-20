use cameleon_impl::memory::memory;

const SBRM_ADDRESS: u64 = 0x1000;
const SIRM_ADDRESS: u64 = 0x2000;
const EIRM_ADDRESS: u64 = 0x3000;

#[memory]
pub struct Memory {
    abrm: register::ABRM,

    sbrm: register::SBRM,
}

mod register {
    use cameleon_impl::memory::register_map;

    #[register_map(base = 0, endianness = LE)]
    pub(super) enum ABRM {
        #[register(len = 2, access = RO, ty = u16)]
        GenCpVersionMinor = 321,

        #[register(len = 2, access = RO, ty = u16)]
        GenCpVersionMajor,

        #[register(len = 64, access = RW, ty = String)]
        ManufacturerName = "Cameleon\0",

        #[register(len = 8, access = RO, ty = u64)]
        SBRMAddress = super::SBRM_ADDRESS,
    }

    #[register_map(base = super::SBRM_ADDRESS, endianness = BE)]
    pub(super) enum SBRM {
        #[register(len = 8, access = RO, ty = u64)]
        SIRMAddress = super::SIRM_ADDRESS,

        #[register(len = 4, access = RO, ty = u32)]
        SIRMLength = 0x20,

        #[register(len = 8, access = RO, ty = u64)]
        EIRMAddress = super::EIRM_ADDRESS,

        #[register(len = 4, access = RO, ty = u32)]
        EIRMLength = 0x20,
    }
}

fn main() {}
