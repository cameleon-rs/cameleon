use cameleon_impl::memory::register;

#[register(base = 0, endianness = LE)]
pub enum ABRM {
    #[entry(len = 2, access = Ro, ty = u16)]
    GenCpVersionMinor = 321,

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
