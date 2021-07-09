/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub trait BitOp: Sized {
    /// Pos is defined as a distance from MSB.
    fn set_bit(self, pos: u8) -> Self;
    /// Pos is defined as a distance from MSB.
    fn clear_bit(self, pos: u8) -> Self;
    /// Pos is defined as a distance from MSB.
    fn is_set(self, pos: u8) -> bool;
}

macro_rules! impl_bit_op{
    ($($ty:ty,)*) => {
        $(
            impl BitOp for $ty {
                fn set_bit(self, pos: u8) -> Self {
                    let num_bits = (std::mem::size_of::<Self>() * 8) as Self;
                    let pos = pos as Self;
                    debug_assert!(pos < num_bits);
                    self | (1 << (num_bits - 1 - pos))
                }

                fn clear_bit(self, pos: u8) -> Self {
                    let num_bits = (std::mem::size_of::<Self>() * 8) as Self;
                    let pos = pos as Self;
                    debug_assert!(pos < num_bits);
                    self & !(1 << (num_bits - 1 - pos))
                }

                fn is_set(self, pos: u8) -> bool {
                    let num_bits = (std::mem::size_of::<Self>() * 8) as Self;
                    let pos = pos as Self;
                    debug_assert!(pos < num_bits);
                    (self >> (num_bits - 1 - pos)) & 1 == 1
                }
            }
        )*
   }
}

impl_bit_op! {
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let x = 0b1111_1111_u8;
        //assert_eq!(x.clear_bit(0), 0b0111_1111);
        assert_eq!(x.clear_bit(4), 0b1111_0111);
        assert_eq!(x.clear_bit(7), 0b1111_1110);

        let x = 0b0000_0000_u8;
        assert_eq!(x.set_bit(0), 0b1000_0000);
        assert_eq!(x.set_bit(3), 0b0001_0000);
        assert_eq!(x.set_bit(7), 0b0000_0001);

        let x = 0b1001_0100_u8;
        assert_eq!(x.is_set(0), true);
        assert_eq!(x.is_set(3), true);
        assert_eq!(x.is_set(4), false);
        assert_eq!(x.is_set(7), false);
    }
}
