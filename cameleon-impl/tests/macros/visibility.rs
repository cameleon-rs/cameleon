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
    use cameleon_impl::memory::register;

    #[register(base = 0, endianess = LE)]
    pub(super) enum ABRM {
        #[entry(len = 2, access = RO, ty = u16)]
        GenCpVersionMinor = 321,

        #[entry(len = 2, access = RO, ty = u16)]
        GenCpVersionMajor,

        #[entry(len = 64, access = RW, ty = String)]
        ManufacturerName = "Cameleon\0",

        #[entry(len = 8, access = RO, ty = u64)]
        SBRMAddress = super::SBRM_ADDRESS,
    }

    #[register(base = super::SBRM_ADDRESS, endianess = BE)]
    pub(super) enum SBRM {
        #[entry(len = 8, access = RO, ty = u64)]
        SIRMAddress = super::SIRM_ADDRESS,

        #[entry(len = 4, access = RO, ty = u32)]
        SIRMLength = 0x20,

        #[entry(len = 8, access = RO, ty = u64)]
        EIRMAddress = super::EIRM_ADDRESS,

        #[entry(len = 4, access = RO, ty = u32)]
        EirmLength = 0x20,
    }
}

fn main() {}
