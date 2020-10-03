use cameleon_impl::memory::register_map;

#[register_map(base = 0, endianness = LE)]
pub enum ABRM {
    #[register(len = 2, access = Ro, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
