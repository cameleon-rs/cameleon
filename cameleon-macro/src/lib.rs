pub use cameleon_macro_impl::register;

use thiserror::Error;

#[doc(hidden)]
pub struct RawEntry {
    pub offset: usize,
    pub len: usize,
}

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("attempt to read unreadable address")]
    AddressNotReadable,

    #[error("attempt to write to unwritable address")]
    AddressNotWritable,

    #[error("attempt to access not existed memory location")]
    InvalidAddress,
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
}

#[doc(hidden)]
pub trait MemoryFragment {
    const SIZE: usize;
    fn fragment() -> (Vec<u8>, MemoryProtection);
}
