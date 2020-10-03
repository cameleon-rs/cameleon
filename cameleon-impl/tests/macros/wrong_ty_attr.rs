use cameleon_impl::memory::register_map;

#[register_map(base = 0, endianness = LE)]
pub enum ABRM {
    #[register(len = 2, access = RO, ty = u32)]
    GenCpVersionMinor = 321,
}

fn main() {}
