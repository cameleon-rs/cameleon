use cameleon_impl::register;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access_right = RO)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access_right = Ro)]
    GenCpVersionMajor,
}

fn main() {}
