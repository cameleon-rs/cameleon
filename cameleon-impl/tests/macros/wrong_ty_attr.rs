use cameleon_impl::memory::register;

#[register(base = 0, endianness = LE)]
pub enum ABRM {
    #[entry(len = 2, access = RO, ty = u32)]
    GenCpVersionMinor = 321,
}

fn main() {}
