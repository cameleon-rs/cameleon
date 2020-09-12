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
    let raw_entry_local = ABRM::GenCpVersionMajor.into_raw_entry_local();
    assert_eq!(raw_entry_local.offset, 2);
    assert_eq!(raw_entry_local.len, 2);
}
