use std::time;

use super::device::RusbDevHandle;
use super::Result;

pub struct ControlChannel {
    device_handle: RusbDevHandle,
    iface_info: ControlIfaceInfo,
}

impl ControlChannel {
    pub(super) fn new(device_handle: RusbDevHandle, iface_info: ControlIfaceInfo) -> Self {
        Self {
            device_handle,
            iface_info,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        self.device_handle
            .claim_interface(self.iface_info.iface_number)?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.device_handle
            .release_interface(self.iface_info.iface_number)?;

        Ok(())
    }

    pub fn send(&self, buf: &[u8], timeout: time::Duration) -> Result<usize> {
        Ok(self
            .device_handle
            .write_bulk(self.iface_info.bulk_out_ep, buf, timeout)?)
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        Ok(self
            .device_handle
            .read_bulk(self.iface_info.bulk_in_ep, buf, timeout)?)
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt(self.iface_info.bulk_in_ep)?;
        self.device_handle.clear_halt(self.iface_info.bulk_out_ep)?;
        Ok(())
    }
}

impl Drop for ControlChannel {
    fn drop(&mut self) {
        if let Err(_err) = self.close() {
            // TODO: logger
            // self.logger.warn(err);
        }
    }
}

pub struct ReceiveChannel {
    device_handle: RusbDevHandle,
    iface_info: ReceiveIfaceInfo,
}

impl ReceiveChannel {
    pub(super) fn new(device_handle: RusbDevHandle, iface_info: ReceiveIfaceInfo) -> Self {
        Self {
            device_handle,
            iface_info,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        self.device_handle
            .claim_interface(self.iface_info.iface_number)?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.device_handle
            .release_interface(self.iface_info.iface_number)?;

        Ok(())
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        Ok(self
            .device_handle
            .read_bulk(self.iface_info.bulk_in_ep, buf, timeout)?)
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt(self.iface_info.bulk_in_ep)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct ControlIfaceInfo {
    pub(super) iface_number: u8,
    pub(super) bulk_in_ep: u8,
    pub(super) bulk_out_ep: u8,
}

#[derive(Clone, Debug)]
pub(super) struct ReceiveIfaceInfo {
    pub(super) iface_number: u8,
    pub(super) bulk_in_ep: u8,
}
