use cameleon_impl::memory::*;

const SBRM_ADDRESS: u64 = 0x1000;

#[register_map(base = 0, endianness = LE)]
pub enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,

    #[register(len = 8, access = RO, ty = u64, offset = 0x1000)]
    TestOffset,

    #[register(len = 8, access = RO, ty = u64)]
    TestOffset2,
}

#[register_map(base = SBRM_ADDRESS, endianness = LE)]
pub enum SBRM {
    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon\0",
}

fn main() {
    assert_eq!(ABRM::SIZE, 0x1008 + 8);

    let raw_reg = ABRM::GenCpVersionMajor::raw();
    assert_eq!(raw_reg.offset, 2);
    assert_eq!(raw_reg.len, 2);

    let mut protection = MemoryProtection::new(ABRM::SIZE);
    ABRM::init_memory_protection(&mut protection);
    assert_eq!(protection.access_right_with_range(0..2), AccessRight::RO);
    assert_eq!(
        protection.access_right_with_range(4..4 + 64),
        AccessRight::RW
    );

    let raw_reg = SBRM::ManufacturerName::raw();
    assert_eq!(raw_reg.offset, SBRM_ADDRESS as usize);
    assert_eq!(raw_reg.len, 64);

    let raw_reg = ABRM::TestOffset::raw();
    assert_eq!(raw_reg.offset, 0x1000);

    let raw_reg = ABRM::TestOffset2::raw();
    assert_eq!(raw_reg.offset, 0x1008);
}
