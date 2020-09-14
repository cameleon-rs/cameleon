use cameleon_impl::memory::register;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access = RO)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access = Ro)]
    GenCpVersionMajor,
}

fn main() {}
