/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{collections::VecDeque, time::Duration};

use nusb::MaybeFuture;
use nusb::{
    transfer::{Buffer, Bulk, ControlOut, EndpointDirection, Recipient},
    Device as NusbDevice,
};

use crate::u3v::{Error, Result, UsbError};
use nusb::transfer::{BulkOrInterrupt, In, Out};

pub struct ControlChannel {
    device: NusbDevice,
    pub iface_info: ControlIfaceInfo,
    state: Option<ControlState>,
}

struct ControlState {
    interface: nusb::Interface,
    bulk_in: nusb::Endpoint<Bulk, In>,
    bulk_out: nusb::Endpoint<Bulk, Out>,
}

impl ControlChannel {
    pub fn new(device: NusbDevice, iface_info: ControlIfaceInfo) -> Result<Self> {
        Ok(Self {
            device,
            iface_info,
            state: None,
        })
    }

    pub fn open(&mut self) -> Result<()> {
        if self.is_opened() {
            return Ok(());
        }

        let interface = self
            .device
            .claim_interface(self.iface_info.iface_number)
            .wait()
            .map_err(Error::from)?;

        // Ensure we are using the default alternate setting.
        interface.set_alt_setting(0).wait().map_err(Error::from)?;

        let bulk_in = interface
            .endpoint::<Bulk, In>(self.iface_info.bulk_in_ep)
            .map_err(Error::from)?;
        let bulk_out = interface
            .endpoint::<Bulk, Out>(self.iface_info.bulk_out_ep)
            .map_err(Error::from)?;

        self.state = Some(ControlState {
            interface,
            bulk_in,
            bulk_out,
        });

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.state.take();
        Ok(())
    }

    #[must_use]
    pub fn is_opened(&self) -> bool {
        self.state.is_some()
    }

    pub fn send(&mut self, buf: &[u8], timeout: Duration) -> Result<usize> {
        let state = self.state.as_mut().ok_or(Error::InvalidDevice)?;
        write_bulk(&mut state.bulk_out, buf, timeout)
    }

    pub fn recv(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        let state = self.state.as_mut().ok_or(Error::InvalidDevice)?;
        read_bulk(&mut state.bulk_in, buf, timeout)
    }

    pub fn set_halt(&mut self, timeout: Duration) -> Result<()> {
        let state = self.state.as_ref().ok_or(Error::InvalidDevice)?;

        send_standard_request(
            &state.interface,
            0x03,
            0x00,
            self.iface_info.bulk_in_ep,
            timeout,
        )?;
        send_standard_request(
            &state.interface,
            0x03,
            0x00,
            self.iface_info.bulk_out_ep,
            timeout,
        )
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        let state = self.state.as_mut().ok_or(Error::InvalidDevice)?;

        state.bulk_in.clear_halt().wait().map_err(Error::from)?;
        state.bulk_out.clear_halt().wait().map_err(Error::from)?;
        Ok(())
    }
}

pub struct ReceiveChannel {
    device: NusbDevice,
    pub iface_info: ReceiveIfaceInfo,
    state: Option<ReceiveState>,
}

struct ReceiveState {
    interface: nusb::Interface,
    endpoint: nusb::Endpoint<Bulk, In>,
}

impl ReceiveChannel {
    pub fn new(device: NusbDevice, iface_info: ReceiveIfaceInfo) -> Result<Self> {
        Ok(Self {
            device,
            iface_info,
            state: None,
        })
    }

    pub fn open(&mut self) -> Result<()> {
        if self.is_opened() {
            return Ok(());
        }

        let interface = self
            .device
            .claim_interface(self.iface_info.iface_number)
            .wait()
            .map_err(Error::from)?;
        interface
            .set_alt_setting(self.iface_info.alt_setting)
            .wait()
            .map_err(Error::from)?;

        let endpoint = interface
            .endpoint::<Bulk, In>(self.iface_info.bulk_in_ep)
            .map_err(Error::from)?;

        self.state = Some(ReceiveState {
            interface,
            endpoint,
        });
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.state.take();
        Ok(())
    }

    #[must_use]
    pub fn is_opened(&self) -> bool {
        self.state.is_some()
    }

    pub fn recv(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        let state = self.state.as_mut().ok_or(Error::InvalidDevice)?;
        read_bulk(&mut state.endpoint, buf, timeout)
    }

    pub fn set_halt(&mut self, timeout: Duration) -> Result<()> {
        let state = self.state.as_ref().ok_or(Error::InvalidDevice)?;
        send_standard_request(
            &state.interface,
            0x03,
            0x00,
            self.iface_info.bulk_in_ep,
            timeout,
        )
    }

    pub fn clear_halt(&mut self) -> Result<()> {
        let state = self.state.as_mut().ok_or(Error::InvalidDevice)?;
        state.endpoint.clear_halt().wait().map_err(Error::from)?;
        Ok(())
    }

    pub fn async_pool(&mut self) -> Result<AsyncPool<'_>> {
        AsyncPool::new(self)
    }
}

#[derive(Clone, Debug)]
pub struct ControlIfaceInfo {
    pub iface_number: u8,
    pub bulk_in_ep: u8,
    pub bulk_out_ep: u8,
}

#[derive(Clone, Debug)]
pub struct ReceiveIfaceInfo {
    pub iface_number: u8,
    pub alt_setting: u8,
    pub bulk_in_ep: u8,
}

fn read_bulk(
    endpoint: &mut nusb::Endpoint<Bulk, In>,
    buf: &mut [u8],
    timeout: Duration,
) -> Result<usize> {
    if buf.is_empty() {
        return Ok(0);
    }

    let requested_len = align_to_packet(buf.len(), endpoint.max_packet_size());
    let mut transfer_buf = endpoint.allocate(requested_len);
    transfer_buf.set_requested_len(requested_len);
    endpoint.submit(transfer_buf);

    match endpoint.wait_next_complete(timeout) {
        Some(completion) => handle_completion(completion, Some(buf)),
        None => {
            endpoint.cancel_all();
            drain_pending(endpoint);
            Err(Error::Usb(UsbError::Timeout))
        }
    }
}

fn write_bulk(
    endpoint: &mut nusb::Endpoint<Bulk, Out>,
    buf: &[u8],
    timeout: Duration,
) -> Result<usize> {
    if buf.is_empty() {
        return Ok(0);
    }

    let mut transfer_buf = Buffer::new(buf.len());
    transfer_buf.extend_from_slice(buf);
    endpoint.submit(transfer_buf);

    match endpoint.wait_next_complete(timeout) {
        Some(completion) => handle_completion(completion, None),
        None => {
            endpoint.cancel_all();
            drain_pending(endpoint);
            Err(Error::Usb(UsbError::Timeout))
        }
    }
}

fn handle_completion(
    completion: nusb::transfer::Completion,
    target: Option<&mut [u8]>,
) -> Result<usize> {
    let actual_len = completion.actual_len;
    let copied_len = if let Some(dst) = target {
        let len = actual_len.min(dst.len());
        dst[..len].copy_from_slice(&completion.buffer[..len]);
        len
    } else {
        actual_len
    };

    match completion.status {
        Ok(()) => Ok(copied_len),
        Err(err) => Err(Error::from(err)),
    }
}

fn send_standard_request(
    interface: &nusb::Interface,
    request: u8,
    value: u16,
    index: u8,
    timeout: Duration,
) -> Result<()> {
    interface
        .control_out(
            ControlOut {
                control_type: nusb::transfer::ControlType::Standard,
                recipient: Recipient::Endpoint,
                request,
                value,
                index: u16::from(index),
                data: &[],
            },
            timeout,
        )
        .wait()
        .map_err(|err| Error::from(err))
}

fn align_to_packet(len: usize, packet_size: usize) -> usize {
    if packet_size == 0 {
        return len.max(1);
    }

    let len = len.max(1);
    let packets = (len + packet_size - 1) / packet_size;
    packets * packet_size
}

fn drain_pending<EpType, Dir>(endpoint: &mut nusb::Endpoint<EpType, Dir>)
where
    EpType: BulkOrInterrupt,
    Dir: EndpointDirection,
{
    const MAX_ATTEMPTS: usize = 8;
    let mut attempts = 0;
    while endpoint.pending() > 0 && attempts < MAX_ATTEMPTS {
        if let Some(completion) = endpoint.wait_next_complete(Duration::from_millis(10)) {
            let _ = completion.status;
            attempts = 0;
        } else {
            attempts += 1;
        }
    }
}

pub struct AsyncPool<'a> {
    endpoint: &'a mut nusb::Endpoint<Bulk, In>,
    pending: VecDeque<PendingTransfer>,
}

struct PendingTransfer {
    target_ptr: *mut u8,
    target_len: usize,
}

impl<'a> AsyncPool<'a> {
    pub fn new(channel: &'a mut ReceiveChannel) -> Result<Self> {
        let state = channel.state.as_mut().ok_or(Error::InvalidDevice)?;
        Ok(Self {
            endpoint: &mut state.endpoint,
            pending: VecDeque::new(),
        })
    }

    pub fn submit(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        let requested_len = align_to_packet(buf.len(), self.endpoint.max_packet_size());
        let mut transfer_buf = self.endpoint.allocate(requested_len);
        transfer_buf.set_requested_len(requested_len);
        self.endpoint.submit(transfer_buf);
        self.pending.push_back(PendingTransfer {
            target_ptr: buf.as_mut_ptr(),
            target_len: buf.len(),
        });
        Ok(())
    }

    pub fn poll(&mut self, timeout: Duration) -> Result<usize> {
        debug_assert!(!self.pending.is_empty());
        let completion = self
            .endpoint
            .wait_next_complete(timeout)
            .ok_or(Error::Usb(UsbError::Timeout))?;

        let pending = self.pending.pop_front().unwrap();

        let len = completion.actual_len.min(pending.target_len);
        unsafe {
            let target_slice = std::slice::from_raw_parts_mut(pending.target_ptr, len);
            target_slice.copy_from_slice(&completion.buffer[..len]);
        }

        match completion.status {
            Ok(()) => Ok(len),
            Err(err) => Err(Error::from(err)),
        }
    }

    pub fn cancel_all(&mut self) {
        self.endpoint.cancel_all();
    }

    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Drop for AsyncPool<'_> {
    fn drop(&mut self) {
        self.cancel_all();
        let mut attempts = 0;
        while !self.is_empty() && attempts < 16 {
            if self.poll(Duration::from_millis(50)).is_ok() {
                attempts = 0;
            } else {
                attempts += 1;
            }
        }
        self.pending.clear();
    }
}
