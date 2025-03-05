/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::upper_case_acronyms)]

pub use cameleon_impl_macros::{memory, register_map};

use thiserror::Error;

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("attempt to read unreadable address")]
    AddressNotReadable,

    #[error("attempt to write to unwritable address")]
    AddressNotWritable,

    #[error("attempt to access non-existent memory location")]
    InvalidAddress,

    #[error("invalid register data: {0}")]
    InvalidRegisterData(std::borrow::Cow<'static, str>),
}

pub mod prelude {
    pub use super::{MemoryRead, MemoryWrite, Register};
}

pub trait MemoryRead {
    fn read_raw(&self, range: std::ops::Range<usize>) -> MemoryResult<&[u8]>;

    fn access_right<T: Register>(&self) -> AccessRight;

    /// Read data from the register.
    /// Since the host side know nothing about `Register`, this method can be called only from the machine side so access rights are temporarily set to `RW`.
    fn read<T: Register>(&self) -> MemoryResult<T::Ty>;
}

pub trait MemoryWrite {
    fn write_raw(&mut self, addr: usize, buf: &[u8]) -> MemoryResult<()>;

    /// Write data to the register.
    /// Since the host side know nothing about `Register`, this method can be called only from the machine side so access rights are temporarily set to `RW`.
    fn write<T: Register>(&mut self, data: T::Ty) -> MemoryResult<()>;

    fn set_access_right<T: Register>(&mut self, access_right: AccessRight);

    fn register_observer<T, U>(&mut self, observer: U)
    where
        T: Register,
        U: MemoryObserver + 'static;
}

pub trait MemoryObserver: Send {
    fn update(&self);
}

/// Represent access right of each memory cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRight {
    /// Not Available.
    NA,
    /// Read Only.
    RO,
    /// Write Only.
    WO,
    /// Read Write.
    RW,
}

impl AccessRight {
    #[must_use]
    pub const fn is_readable(self) -> bool {
        self.as_num() & 0b1 == 1
    }

    #[must_use]
    pub const fn is_writable(self) -> bool {
        self.as_num() >> 1_i32 == 1
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NA => "NA",
            Self::RO => "RO",
            Self::WO => "WO",
            Self::RW => "RW",
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn as_num(self) -> u8 {
        match self {
            Self::NA => 0b00,
            Self::RO => 0b01,
            Self::WO => 0b10,
            Self::RW => 0b11,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn meet(self, rhs: Self) -> Self {
        use AccessRight::{NA, RO, RW, WO};
        match self {
            RW => {
                if rhs == RW {
                    RW
                } else {
                    rhs
                }
            }
            RO => {
                if rhs.is_readable() {
                    self
                } else {
                    NA
                }
            }
            WO => {
                if rhs.is_writable() {
                    self
                } else {
                    NA
                }
            }
            NA => NA,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn from_num(num: u8) -> Self {
        debug_assert!(num >> 2_i32 == 0);
        match num {
            0b00 => Self::NA,
            0b01 => Self::RO,
            0b10 => Self::WO,
            0b11 => Self::RW,
            _ => unreachable!(),
        }
    }
}

#[doc(hidden)]
pub struct MemoryProtection {
    inner: Vec<u8>,
    memory_size: usize,
}

impl MemoryProtection {
    #[must_use]
    pub fn new(memory_size: usize) -> Self {
        let len = if memory_size == 0 {
            0
        } else {
            (memory_size - 1) / 4 + 1
        };
        let inner = vec![0; len];
        Self { inner, memory_size }
    }

    pub fn set_access_right(&mut self, address: usize, access_right: AccessRight) {
        let block = &mut self.inner[address / 4];
        let offset = address % 4 * 2;
        let mask = !(0b11 << offset);
        *block = (*block & mask) | (access_right.as_num() << offset);
    }

    #[must_use]
    pub fn access_right(&self, address: usize) -> AccessRight {
        let block = self.inner[address / 4];
        let offset = address % 4 * 2;
        AccessRight::from_num((block >> offset) & 0b11)
    }

    pub fn access_right_with_range(&self, range: impl IntoIterator<Item = usize>) -> AccessRight {
        range
            .into_iter()
            .fold(AccessRight::RW, |acc, i| acc.meet(self.access_right(i)))
    }

    pub fn set_access_right_with_range(
        &mut self,
        range: impl IntoIterator<Item = usize>,
        access_right: AccessRight,
    ) {
        range
            .into_iter()
            .for_each(|i| self.set_access_right(i, access_right));
    }

    pub fn verify_address(&self, address: usize) -> MemoryResult<()> {
        if self.memory_size <= address {
            Err(MemoryError::InvalidAddress)
        } else {
            Ok(())
        }
    }

    pub fn verify_address_with_range(
        &self,
        range: impl IntoIterator<Item = usize>,
    ) -> MemoryResult<()> {
        for i in range {
            self.verify_address(i)?;
        }
        Ok(())
    }
}

pub trait Register {
    type Ty;

    const ADDRESS: usize;
    const LENGTH: usize;
    const ACCESS_RIGHT: AccessRight;

    fn parse(data: &[u8]) -> MemoryResult<Self::Ty>;
    fn serialize(data: Self::Ty) -> MemoryResult<Vec<u8>>;

    fn write(data: Self::Ty, memory: &mut [u8]) -> MemoryResult<()> {
        let data = Self::serialize(data)?;
        let range = Self::range();

        memory[range].copy_from_slice(data.as_slice());
        Ok(())
    }

    fn read(memory: &[u8]) -> MemoryResult<Self::Ty> {
        let range = Self::range();
        Self::parse(&memory[range])
    }

    #[must_use]
    fn range() -> std::ops::Range<usize> {
        Self::ADDRESS..Self::ADDRESS + Self::LENGTH
    }
}

#[cfg(test)]
mod tests {
    use super::AccessRight::{NA, RO, RW, WO};
    use super::*;

    #[test]
    fn test_protection() {
        // [RO, RW, NA, WO, RO];
        let mut protection = MemoryProtection::new(5);
        protection.set_access_right(0, RO);
        protection.set_access_right(1, RW);
        protection.set_access_right(2, NA);
        protection.set_access_right(3, WO);
        protection.set_access_right(4, RO);

        assert_eq!(protection.inner.len(), 2);
        assert_eq!(protection.access_right(0), RO);
        assert_eq!(protection.access_right(1), RW);
        assert_eq!(protection.access_right(2), NA);
        assert_eq!(protection.access_right(3), WO);
        assert_eq!(protection.access_right(4), RO);

        assert_eq!(protection.access_right_with_range(0..2), RO);
        assert_eq!(protection.access_right_with_range(2..4), NA);
        assert_eq!(protection.access_right_with_range(3..5), NA);
    }

    #[test]
    fn test_verify_address() {
        let protection = MemoryProtection::new(5);
        assert!(protection.verify_address(0).is_ok());
        assert!(protection.verify_address(4).is_ok());
        assert!(protection.verify_address(5).is_err());
        assert!(protection.verify_address_with_range(2..5).is_ok());
        assert!(protection.verify_address_with_range(2..6).is_err());
    }
}
