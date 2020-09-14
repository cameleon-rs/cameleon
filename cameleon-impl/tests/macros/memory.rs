use cameleon_impl::{memory, prelude::*, register};

const SBRM_ADDRESS: u64 = 0x1000;
const SIRM_ADDRESS: u64 = 0x2000;
const EIRM_ADDRESS: u64 = 0x3000;

#[memory]
pub struct Memory {
    #[offset(0)]
    abrm: ABRM,

    #[offset(SBRM_ADDRESS)]
    sbrm: SBRM,
}

#[register(endianess = LE)]
pub enum ABRM {
    #[entry(len = 2, access = RO)]
    GenCpVersionMinor = 321,

    #[entry(len = 2, access = RO)]
    GenCpVersionMajor,

    #[entry(len = 64, access = RW)]
    ManufacturerName = "Cameleon\0",

    #[entry(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

#[register(endianess = LE)]
pub enum SBRM {
    #[entry(len = 8, access = RO, ty = u64)]
    SIRMAddress = SIRM_ADDRESS,

    #[entry(len = 4, access = RO)]
    SIRMLength = 0x20,

    #[entry(len = 8, access = RO, ty = u64)]
    EIRMAddress = EIRM_ADDRESS,

    #[entry(len = 4, access = RO)]
    EirmLength = 0x20,
}

fn main() {
    use cameleon_impl::byteorder::{ReadBytesExt, LE};

    let mut memory = Memory::new();
    let mut gen_cp_minor = memory.read_entry(ABRM::GenCpVersionMinor).unwrap();
    assert_eq!(gen_cp_minor.len(), 2);
    assert_eq!(gen_cp_minor.read_u16::<LE>().unwrap(), 321);

    let mut sbrm_address = memory.read_entry(ABRM::SBRMAddress).unwrap();
    let sbrm_address = sbrm_address.read_u64::<LE>().unwrap();
    assert_eq!(sbrm_address, SBRM_ADDRESS);

    let mut sirm_address = memory
        .read(sbrm_address as usize..sbrm_address as usize + 8)
        .unwrap();
    assert_eq!(sirm_address, memory.read_entry(SBRM::SIRMAddress).unwrap());

    let sirm_address = sirm_address.read_u64::<LE>().unwrap();
    assert_eq!(sirm_address, SIRM_ADDRESS);

    assert_eq!(
        memory.access_right(SBRM::EirmLength),
        cameleon_impl::AccessRight::RO
    );
    memory.set_access_right(SBRM::EirmLength, cameleon_impl::AccessRight::NA);
    assert_eq!(
        memory.access_right(SBRM::EirmLength),
        cameleon_impl::AccessRight::NA
    );

    assert!(memory.read(1000..1004).is_err());
}
