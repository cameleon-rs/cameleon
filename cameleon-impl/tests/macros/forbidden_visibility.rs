const SBRM_ADDRESS: u64 = 0x1000;

mod outer {
    pub(super) mod inner {
        use cameleon_impl::memory::register;

        #[register(base = 0, endianness = LE)]
        pub(in super::super) enum ABRM {
            #[entry(len = 2, access = RO, ty = u16)]
            GenCpVersionMinor = 321,

            #[entry(len = 2, access = RO, ty = u16)]
            GenCpVersionMajor,

            #[entry(len = 64, access = RW, ty = String)]
            ManufacturerName = "Cameleon\0",

            #[entry(len = 8, access = RO, ty = u64)]
            SBRMAddress = super::super::SBRM_ADDRESS,
        }
    }
}

fn main() {}
