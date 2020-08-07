use std::borrow::Cow;
use std::io::Write;

use super::{EmulatorError, Result};
use semver::Version;

pub struct Memory {
    raw: Vec<u8>,
}

impl Memory {
    pub(super) fn new(size: usize) -> Self {
        let raw = vec![0; size];
        let mut memory = Self { raw };
        memory
    }

    pub(super) fn from_raw(raw: Vec<u8>) -> Self {
        Self { raw }
    }

    pub(super) fn dump(&self) -> &[u8] {
        &self.raw
    }

    pub(super) fn read(&mut self, address: u64) -> Result<u8> {
        let address = address as usize;
        self.check_access(address)?;
        Ok(self.raw[address])
    }

    pub(super) fn write(&mut self, address: u64, data: u8) -> Result<()> {
        let address = address as usize;
        self.check_access(address)?;

        self.raw[address] = data;
        Ok(())
    }

    fn check_access(&self, address: usize) -> Result<()> {
        if self.raw.len() < address {
            Err(EmulatorError::MemoryAccessViolation)
        } else {
            Ok(())
        }
    }
}

