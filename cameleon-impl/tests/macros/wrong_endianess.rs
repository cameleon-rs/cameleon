use cameleon_impl::register;

//const SBRM_ADDRESS: u64 = 0x1000;

#[register(endianess = Be)]
pub enum ABRM {
    #[entry(len = 2, access = RO)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access = RW)]
    ManufacturerName = "Cameleon",

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
