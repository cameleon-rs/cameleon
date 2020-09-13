pub use byteorder;
pub use cameleon_impl_macros::{memory, register};

use thiserror::Error;

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("attempt to read unreadable address")]
    AddressNotReadable,

    #[error("attempt to write to unwritable address")]
    AddressNotWritable,

    #[error("attempt to access not existed memory location")]
    InvalidAddress,

    #[error("attemt to write bytes to entry, but the bytes length is greater than entry length")]
    EntryOverrun,
}

pub mod prelude {
    pub use super::{MemoryRead, MemoryWrite};
}

pub trait MemoryRead {
    fn read(&self, range: std::ops::Range<usize>) -> MemoryResult<&[u8]>;
    fn access_right(&self, entry: impl Into<RawEntry>) -> AccessRight;
    fn read_entry(&self, entry: impl Into<RawEntry>) -> MemoryResult<&[u8]>;
}

pub trait MemoryWrite {
    fn write(&mut self, addr: usize, buf: &[u8]) -> MemoryResult<()>;
    fn write_entry(&mut self, entry: impl Into<RawEntry>, buf: &[u8]) -> MemoryResult<()>;
    fn set_access_right(&mut self, entry: impl Into<RawEntry>, access_right: AccessRight);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRight {
    NA,
    RO,
    WO,
    RW,
}

impl AccessRight {
    pub fn is_readable(self) -> bool {
        self.as_num() & 0b1 == 1
    }

    pub fn is_writable(self) -> bool {
        self.as_num() >> 1 == 1
    }

    #[doc(hidden)]
    pub fn as_num(self) -> u8 {
        match self {
            Self::NA => 0b00,
            Self::RO => 0b01,
            Self::WO => 0b10,
            Self::RW => 0b11,
        }
    }

    #[doc(hidden)]
    pub fn meet(self, rhs: Self) -> Self {
        use AccessRight::*;
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
    pub fn from_num(num: u8) -> Self {
        debug_assert!(num >> 2 == 0);
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
        *block = (*block & mask) | access_right.as_num() << offset;
    }

    pub fn access_right(&self, address: usize) -> AccessRight {
        let block = self.inner[address / 4];
        let offset = address % 4 * 2;
        AccessRight::from_num(block >> offset & 0b11)
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

    pub fn copy_from(&mut self, rhs: &Self, offset: usize) {
        // Super slow operation.
        // TODO: use bitwise operation to copy.
        for i in 0..rhs.memory_size {
            let access_right = rhs.access_right(i);
            self.set_access_right(offset + i, access_right);
        }
    }
}

#[doc(hidden)]
pub trait MemoryFragment {
    const SIZE: usize;

    fn fragment() -> Vec<u8>;
    fn memory_protection() -> MemoryProtection;
    fn local_raw_entry(&self) -> RawEntry;
}

#[doc(hidden)]
pub struct RawEntry {
    pub offset: usize,
    pub len: usize,
}

impl RawEntry {
    pub fn new(offset: usize, len: usize) -> Self {
        Self { offset, len }
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        let start = self.offset;
        let end = start + self.len;
        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::AccessRight::*;
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
