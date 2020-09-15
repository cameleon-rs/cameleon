use cameleon_impl::memory::register;

#[register(base = 0, endianness = LE)]
enum ABRM {
    #[entry(len = 4, access = RO, ty = Bytes)]
    ProtocolEndianness = &[0xFF, 1000, 0xFF, 0xFF],
}

fn main() {}
