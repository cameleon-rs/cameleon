use cameleon_impl::memory::*;

const SBRM_ADDRESS: u64 = 0x1000;

#[register_map(base = 0, endianness = LE)]
pub enum ABRM {
    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 2, access = RO, ty = u16)]
    GenCpVersionMajor,

    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon",

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,

    #[register(len = 8, access = RO, ty = u64, offset = 0x1000)]
    TestOffset,

    #[register(len = 8, access = RO, ty = u64)]
    TestOffset2,
}

const MODEL_NAME_LEN: usize = 64;

#[register_map(base = SBRM_ADDRESS, endianness = LE)]
pub enum SBRM {
    #[register(len = 64, access = RW, ty = String)]
    ManufacturerName = "Cameleon",

    #[register(len = MODEL_NAME_LEN, access = RW, ty = String)]
    ModelName = "Cameleon Model",
}

fn main() {
    assert_eq!(ABRM::size(), 0x1008 + 8);

    let (addr, len) = (
        ABRM::GenCpVersionMajor::ADDRESS,
        ABRM::GenCpVersionMajor::LENGTH,
    );
    assert_eq!(addr, 2);
    assert_eq!(len, 2);

    let mut protection = MemoryProtection::new(ABRM::size());
    ABRM::init_memory_protection(&mut protection);
    assert_eq!(protection.access_right_with_range(0..2), AccessRight::RO);
    assert_eq!(
        protection.access_right_with_range(4..4 + 64),
        AccessRight::RW
    );

    let (addr, len) = (
        SBRM::ManufacturerName::ADDRESS,
        SBRM::ManufacturerName::LENGTH,
    );
    assert_eq!(addr, SBRM_ADDRESS as usize);
    assert_eq!(len, 64);

    let (addr, len) = (SBRM::ModelName::ADDRESS, SBRM::ModelName::LENGTH);
    assert_eq!(addr, SBRM_ADDRESS as usize + 64,);
    assert_eq!(len, MODEL_NAME_LEN);

    let addr = ABRM::TestOffset::ADDRESS;
    assert_eq!(addr, 0x1000);

    let addr = ABRM::TestOffset2::ADDRESS;
    assert_eq!(addr, 0x1008);
}
