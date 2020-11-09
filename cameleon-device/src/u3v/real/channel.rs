use std::time;

use crate::u3v::Result;

use super::device::RusbDevHandle;

pub struct ControlChannel {
    device_handle: RusbDevHandle,
    iface_info: ControlIfaceInfo,
    is_open: bool,
}

impl ControlChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_open() {
            self.device_handle
                .claim_interface(self.iface_info.iface_number)?;
            self.is_open = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_open() {
            self.device_handle
                .release_interface(self.iface_info.iface_number)?;
            self.is_open = false;
        }

        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open
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

    pub fn set_halt(&self, timeout: time::Duration) -> Result<()> {
        set_halt(&self.device_handle, self.iface_info.bulk_in_ep, timeout)?;
        set_halt(&self.device_handle, self.iface_info.bulk_out_ep, timeout)?;

        Ok(())
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt(self.iface_info.bulk_in_ep)?;
        self.device_handle.clear_halt(self.iface_info.bulk_out_ep)?;
        Ok(())
    }
    pub(super) fn new(device_handle: RusbDevHandle, iface_info: ControlIfaceInfo) -> Self {
        Self {
            device_handle,
            iface_info,
            is_open: false,
        }
    }
}

pub struct ReceiveChannel {
    device_handle: RusbDevHandle,
    iface_info: ReceiveIfaceInfo,
    is_open: bool,
}

impl ReceiveChannel {
    pub fn open(&mut self) -> Result<()> {
        if !self.is_open() {
            self.device_handle
                .claim_interface(self.iface_info.iface_number)?;
            self.is_open = true;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.is_open() {
            self.device_handle
                .release_interface(self.iface_info.iface_number)?;
        }

        self.is_open = false;
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn recv(&self, buf: &mut [u8], timeout: time::Duration) -> Result<usize> {
        Ok(self
            .device_handle
            .read_bulk(self.iface_info.bulk_in_ep, buf, timeout)?)
    }

    pub fn set_halt(&self, timeout: time::Duration) -> Result<()> {
        set_halt(&self.device_handle, self.iface_info.bulk_in_ep, timeout)?;

        Ok(())
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        self.device_handle.clear_halt(self.iface_info.bulk_in_ep)?;
        Ok(())
    }

    pub(super) fn new(device_handle: RusbDevHandle, iface_info: ReceiveIfaceInfo) -> Self {
        Self {
            device_handle,
            iface_info,
            is_open: false,
        }
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

fn set_halt(handle: &RusbDevHandle, endpoint_number: u8, timeout: time::Duration) -> Result<()> {
    let request_type = rusb::request_type(
        rusb::Direction::Out,
        rusb::RequestType::Standard,
        rusb::Recipient::Endpoint,
    );
    let request = 0x03; // SET_FEATURE.
    let value = 0x00; // ENDPOINT_HALT.
    let buf = vec![]; // NO DATA.

    handle.write_control(
        request_type,
        request,
        value,
        endpoint_number as u16,
        &buf,
        timeout,
    )?;

    Ok(())
}
