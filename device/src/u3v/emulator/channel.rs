use std::time;

use crate::u3v::Result;

use super::emulator_impl::DeviceHandle;

pub struct ControlChannel {
    device_handle: DeviceHandle,
    is_opened: bool,
}

impl ControlChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_opened() {
            self.device_handle.claim_interface()?;
            self.is_opened = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_opened() {
            self.device_handle.release_interface()?;
            self.is_opened = false;
        }

        Ok(())
    }

    #[must_use]
    pub fn is_opened(&self) -> bool {
        self.is_opened
    }

    pub fn send(&self, buf: &[u8], timeout: time::Duration) -> Result<usize> {
        self.device_handle.write_bulk(buf, timeout)
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        self.device_handle.read_bulk(buf, timeout)
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
            is_opened: false,
        }
    }
}

impl Drop for ControlChannel {
    // TODO: logging.
    fn drop(&mut self) {
        let _res = self.close();
    }
}

pub struct ReceiveChannel {
    device_handle: DeviceHandle,
    is_opened: bool,
}

impl ReceiveChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_opened() {
            self.device_handle.claim_interface()?;
            self.is_opened = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_opened() {
            self.device_handle.release_interface()?;
        }

        self.is_opened = false;
        Ok(())
    }

    #[must_use]
    pub fn is_opened(&self) -> bool {
        self.is_opened
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        self.device_handle.read_bulk(buf, timeout)
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
            is_opened: false,
        }
    }
}

impl Drop for ReceiveChannel {
    fn drop(&mut self) {
        let _res = self.close();
    }
}
