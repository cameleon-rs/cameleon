use cameleon_impl::memory::register_map;

const SBRM_ADDRESS: u64 = 0x1000;

#[register_map(base = 0, endianness = Le)]
pub enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
