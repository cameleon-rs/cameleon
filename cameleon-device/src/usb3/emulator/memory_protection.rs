use super::{EmulatorError, EmulatorResult};
use crate::usb3::register_map::AccessRight;

/// Map each address to its access right.
/// Access right is represented by 2 bits and mapping is done in 2 steps described below.
/// 1. First step is calculating block corresponding to the address. 4 access rights is packed into a single block, thus the block
///    position is calculated by `address / 4`.
/// 2. Second step is extracting the access right from the block. The offset of the access right is calculated by
///    `address % 4 * 2`.
// TODO: Consider better representation.
pub(super) struct MemoryProtection {
    inner: Vec<u8>,
    memory_size: usize,
}

impl MemoryProtection {
    pub(super) fn new(memory_size: usize) -> Self {
        let len = if memory_size == 0 {
            0
        } else {
            (memory_size - 1) / 4 + 1
        };
        let inner = vec![0; len];
        Self { inner, memory_size }
    }

    pub(super) fn set_access_right(&mut self, address: usize, access_right: AccessRight) {
        let block = &mut self.inner[address / 4];
        let offset = address % 4 * 2;
        let mask = !(0b11 << offset);
        *block = (*block & mask) | access_right.as_num() << offset;
    }

    pub(super) fn access_right(&self, address: usize) -> AccessRight {
        let block = self.inner[address / 4];
        let offset = address % 4 * 2;
        AccessRight::from_num(block >> offset & 0b11)
    }

    pub(super) fn verify_address(&self, address: usize) -> EmulatorResult<()> {
        if self.memory_size <= address {
            Err(EmulatorError::InvalidAddress)
        } else {
            Ok(())
        }
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
    }

    #[test]
    fn test_verify_address() {
        let protection = MemoryProtection::new(5);
        assert!(protection.verify_address(0).is_ok());
        assert!(protection.verify_address(4).is_ok());
        assert!(protection.verify_address(5).is_err());
    }
}
