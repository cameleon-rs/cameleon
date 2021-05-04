use cameleon_impl::memory::*;

#[memory]
pub struct Memory {
    le: LE,
    be: BE,
}

#[register_map(base = 0, endianness = LE)]
pub enum LE {
    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 1, MSB = 1>)]
    U8Bit = 0b1,

    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 1, MSB = 4>)]
    U8 = 0b1011,

    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 0, MSB = 7>)]
    U8Full = 0xff,

    #[register(len = 1, access = RO, ty = BitField<i8, LSB = 1, MSB = 4>)]
    I8 = -3,

    #[register(len = 1, access = RO, ty = BitField<i8, LSB = 0, MSB = 7>)]
    I8Full = -128,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 14, MSB = 14>)]
    U32Bit = 0b1,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 9, MSB = 21>)]
    U32 = 0b1_0010_1110_1101,

    #[register(len = 4, access = RO, ty = BitField<i32, LSB = 9, MSB = 21>)]
    I32 = -324,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 0, MSB = 4>, offset=20)]
    OverlapUnsigned1 = 0b1_0011,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 5, MSB = 9>, offset=20)]
    OverlapUnsigned2 = 0b0_1000,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 10, MSB = 15>, offset=20)]
    OverlapUnsigned3 = 0b11_1111,

    #[register(len = 2, access = RO, ty = BitField<i16, LSB = 0, MSB = 10>, offset=22)]
    OverlapSigned1 = -1,

    #[register(len = 2, access = RO, ty = BitField<i16, LSB = 11, MSB = 15>, offset=22)]
    OverlapSigned2 = 0,
}

#[register_map(base = 100, endianness = BE)]
pub enum BE {
    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 1, MSB = 1>)]
    U8Bit = 0b1,

    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 4, MSB = 1>)]
    U8 = 0b1011,

    #[register(len = 1, access = RO, ty = BitField<u8, LSB = 7, MSB = 0>)]
    U8Full = 0xff,

    #[register(len = 1, access = RO, ty = BitField<i8, LSB = 4, MSB = 1>)]
    I8 = -3,

    #[register(len = 1, access = RO, ty = BitField<i8, LSB = 7, MSB = 0>)]
    I8Full = -128,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 14, MSB = 14>)]
    U32Bit = 0b1,

    #[register(len = 4, access = RO, ty = BitField<u32, LSB = 21, MSB = 9>)]
    U32 = 0b1_0010_1110_1101,

    #[register(len = 4, access = RO, ty = BitField<i32, LSB = 21, MSB = 9>)]
    I32 = -324,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 4, MSB = 0>, offset=20)]
    OverlapUnsigned1 = 0b1_0011,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 9, MSB = 5>, offset=20)]
    OverlapUnsigned2 = 0b0_1000,

    #[register(len = 2, access = RO, ty = BitField<u16, LSB = 15, MSB = 10>, offset=20)]
    OverlapUnsigned3 = 0b11_1111,

    #[register(len = 2, access = RO, ty = BitField<i16, LSB = 10, MSB = 0>, offset=22)]
    OverlapSigned1 = -1,

    #[register(len = 2, access = RO, ty = BitField<i16, LSB = 15, MSB = 11>, offset=22)]
    OverlapSigned2 = 0,
}

fn main() {
    // Test LE.
    assert_eq!(LE::size(), 22 + 2);

    let mut memory = Memory::new();
    assert_eq!(memory.read::<LE::U8Bit>().unwrap(), 0b1);
    memory.write::<LE::U8Bit>(0).unwrap();
    assert_eq!(memory.read::<LE::U8Bit>().unwrap(), 0b0);
    assert_eq!(LE::U8Bit::LSB, 1);
    assert_eq!(LE::U8Bit::MSB, 1);

    assert_eq!(memory.read::<LE::U8>().unwrap(), 0b1011);
    memory.write::<LE::U8>(0b0110).unwrap();
    assert_eq!(memory.read::<LE::U8>().unwrap(), 0b0110);
    assert_eq!(LE::U8::LSB, 1);
    assert_eq!(LE::U8::MSB, 4);

    assert_eq!(memory.read::<LE::U8Full>().unwrap(), 0xff);
    memory.write::<LE::U8Full>(0xb3).unwrap();
    assert_eq!(memory.read::<LE::U8Full>().unwrap(), 0xb3);

    assert_eq!(memory.read::<LE::I8>().unwrap(), -3);
    memory.write::<LE::I8>(4).unwrap();
    assert_eq!(memory.read::<LE::I8>().unwrap(), 4);

    assert_eq!(memory.read::<LE::I8Full>().unwrap(), -128);
    memory.write::<LE::I8Full>(127).unwrap();
    assert_eq!(memory.read::<LE::I8Full>().unwrap(), 127);

    assert_eq!(memory.read::<LE::U32Bit>().unwrap(), 0b1);
    memory.write::<LE::U32Bit>(0).unwrap();
    assert_eq!(memory.read::<LE::U32Bit>().unwrap(), 0b0);

    assert_eq!(memory.read::<LE::U32>().unwrap(), 0b1_0010_1110_1101);
    memory.write::<LE::U32>(0b0_0100_0011_1000).unwrap();
    assert_eq!(memory.read::<LE::U32>().unwrap(), 0b0_0100_0011_1000);

    assert_eq!(memory.read::<LE::I32>().unwrap(), -324);
    memory.write::<LE::I32>(241).unwrap();
    assert_eq!(memory.read::<LE::I32>().unwrap(), 241);

    assert_eq!(memory.read::<LE::OverlapUnsigned1>().unwrap(), 0b1_0011);
    assert_eq!(memory.read::<LE::OverlapUnsigned2>().unwrap(), 0b0_1000);
    assert_eq!(memory.read::<LE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<LE::OverlapUnsigned1>(0b1_1100).unwrap();
    assert_eq!(memory.read::<LE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<LE::OverlapUnsigned2>().unwrap(), 0b0_1000);
    assert_eq!(memory.read::<LE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<LE::OverlapUnsigned2>(0b0_0111).unwrap();
    assert_eq!(memory.read::<LE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<LE::OverlapUnsigned2>().unwrap(), 0b0_0111);
    assert_eq!(memory.read::<LE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<LE::OverlapUnsigned3>(0b00_0000).unwrap();
    assert_eq!(memory.read::<LE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<LE::OverlapUnsigned2>().unwrap(), 0b0_0111);
    assert_eq!(memory.read::<LE::OverlapUnsigned3>().unwrap(), 0b00_0000);

    assert_eq!(memory.read::<LE::OverlapSigned1>().unwrap(), -1);
    assert_eq!(memory.read::<LE::OverlapSigned2>().unwrap(), 0);

    memory.write::<LE::OverlapSigned1>(103).unwrap();
    assert_eq!(memory.read::<LE::OverlapSigned1>().unwrap(), 103);
    assert_eq!(memory.read::<LE::OverlapSigned2>().unwrap(), 0);

    memory.write::<LE::OverlapSigned2>(-1).unwrap();
    assert_eq!(memory.read::<LE::OverlapSigned1>().unwrap(), 103);
    assert_eq!(memory.read::<LE::OverlapSigned2>().unwrap(), -1);

    // Test BE.
    assert_eq!(BE::size(), 22 + 2);

    let mut memory = Memory::new();
    assert_eq!(memory.read::<BE::U8Bit>().unwrap(), 0b1);
    memory.write::<BE::U8Bit>(0).unwrap();
    assert_eq!(memory.read::<BE::U8Bit>().unwrap(), 0b0);

    assert_eq!(memory.read::<BE::U8>().unwrap(), 0b1011);
    memory.write::<BE::U8>(0b0110).unwrap();
    assert_eq!(memory.read::<BE::U8>().unwrap(), 0b0110);
    assert_eq!(BE::U8::LSB, 4);
    assert_eq!(BE::U8::MSB, 1);

    assert_eq!(memory.read::<BE::U8Full>().unwrap(), 0xff);
    memory.write::<BE::U8Full>(0xb3).unwrap();
    assert_eq!(memory.read::<BE::U8Full>().unwrap(), 0xb3);

    assert_eq!(memory.read::<BE::I8>().unwrap(), -3);
    memory.write::<BE::I8>(4).unwrap();
    assert_eq!(memory.read::<BE::I8>().unwrap(), 4);

    assert_eq!(memory.read::<BE::I8Full>().unwrap(), -128);
    memory.write::<BE::I8Full>(127).unwrap();
    assert_eq!(memory.read::<BE::I8Full>().unwrap(), 127);

    assert_eq!(memory.read::<BE::U32Bit>().unwrap(), 0b1);
    memory.write::<BE::U32Bit>(0).unwrap();
    assert_eq!(memory.read::<BE::U32Bit>().unwrap(), 0b0);

    assert_eq!(memory.read::<BE::U32>().unwrap(), 0b1_0010_1110_1101);
    memory.write::<BE::U32>(0b0_0100_0011_1000).unwrap();
    assert_eq!(memory.read::<BE::U32>().unwrap(), 0b0_0100_0011_1000);

    assert_eq!(memory.read::<BE::I32>().unwrap(), -324);
    memory.write::<BE::I32>(241).unwrap();
    assert_eq!(memory.read::<BE::I32>().unwrap(), 241);

    assert_eq!(memory.read::<BE::OverlapUnsigned1>().unwrap(), 0b1_0011);
    assert_eq!(memory.read::<BE::OverlapUnsigned2>().unwrap(), 0b0_1000);
    assert_eq!(memory.read::<BE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<BE::OverlapUnsigned1>(0b1_1100).unwrap();
    assert_eq!(memory.read::<BE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<BE::OverlapUnsigned2>().unwrap(), 0b0_1000);
    assert_eq!(memory.read::<BE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<BE::OverlapUnsigned2>(0b0_0111).unwrap();
    assert_eq!(memory.read::<BE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<BE::OverlapUnsigned2>().unwrap(), 0b0_0111);
    assert_eq!(memory.read::<BE::OverlapUnsigned3>().unwrap(), 0b11_1111);

    memory.write::<BE::OverlapUnsigned3>(0b00_0000).unwrap();
    assert_eq!(memory.read::<BE::OverlapUnsigned1>().unwrap(), 0b1_1100);
    assert_eq!(memory.read::<BE::OverlapUnsigned2>().unwrap(), 0b0_0111);
    assert_eq!(memory.read::<BE::OverlapUnsigned3>().unwrap(), 0b00_0000);

    assert_eq!(memory.read::<BE::OverlapSigned1>().unwrap(), -1);
    assert_eq!(memory.read::<BE::OverlapSigned2>().unwrap(), 0);

    memory.write::<BE::OverlapSigned1>(103).unwrap();
    assert_eq!(memory.read::<BE::OverlapSigned1>().unwrap(), 103);
    assert_eq!(memory.read::<BE::OverlapSigned2>().unwrap(), 0);

    memory.write::<BE::OverlapSigned2>(-1).unwrap();
    assert_eq!(memory.read::<BE::OverlapSigned1>().unwrap(), 103);
    assert_eq!(memory.read::<BE::OverlapSigned2>().unwrap(), -1);
}
