use cameleon_macro::register;

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access_right = RO)]
    GenCpVersionMinor = 1,

    #[entry(len = 2, access_right = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access_right = RO)]
    ManufacturerName = "Cameleon",
}

fn main() {
    let _ = ABRM::GenCpVersionMajor;
}
