use std::time;

use crate::u3v::Result;

use super::emulator_impl::DeviceHandle;

pub struct ControlChannel {
    device_handle: DeviceHandle,
    is_open: bool,
}

impl ControlChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_open() {
            self.device_handle.claim_interface()?;
            self.is_open = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_open() {
            self.device_handle.release_interface()?;
            self.is_open = false;
        }

        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn send(&self, buf: &[u8], timeout: time::Duration) -> Result<usize> {
        Ok(self.device_handle.write_bulk(buf, timeout)?)
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        Ok(self.device_handle.read_bulk(buf, timeout)?)
    }

    pub fn set_halt(&self, _timeout: time::Duration) -> Result<()> {
        // Set halt timeout isn't suppoted.
        self.device_handle.set_halt()
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt()
    }

    pub(super) fn new(device_handle: DeviceHandle) -> Self {
        Self {
            device_handle,
            is_open: false,
        }
    }
}

pub struct ReceiveChannel {
    device_handle: DeviceHandle,
    is_open: bool,
}

impl ReceiveChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_open() {
            self.device_handle.claim_interface()?;
            self.is_open = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_open() {
            self.device_handle.release_interface()?;
        }

        self.is_open = false;
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        Ok(self.device_handle.read_bulk(buf, timeout)?)
    }

    pub fn set_halt(&self, _timeout: time::Duration) -> Result<()> {
        // Set halt timeout isn't suppoted.
        self.device_handle.set_halt()
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt()
    }

    pub(super) fn new(device_handle: DeviceHandle) -> Self {
        Self {
            device_handle,
            is_open: false,
        }
    }
}
