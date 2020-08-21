use std::{collections::HashMap, sync::Arc};

use async_std::sync::{Mutex, Receiver, Sender};
use lazy_static::lazy_static;

use crate::usb3::{LibUsbError, Result};

use super::{
    device::Device,
    fake_protocol::{FakeAckPacket, FakeReqPacket, IfaceKind},
};

lazy_static! {
    pub(crate) static ref DEVICE_POOL: Arc<Mutex<DevicePool>> =
        Arc::new(Mutex::new(DevicePool::new()));
}

pub(crate) struct DevicePool {
    contexts: Vec<Context>,
    next_id: u32,
}

impl DevicePool {
    pub(super) fn claim_interface(
        &mut self,
        device_id: u32,
        iface: IfaceKind,
    ) -> Result<Arc<Mutex<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)>>> {
        self.ctx_mut(device_id)?.claim_interface(iface)
    }

    pub(super) fn release_interface(&mut self, device_id: u32, iface: IfaceKind) -> Result<()> {
        let ctx = self.ctx_mut(device_id)?;
        ctx.release_interface(iface);
        Ok(())
    }

    pub(super) fn pool_and_run(&mut self, device: Device) {
        let ctx = Context::run(device, self.next_id);

        self.next_id += 1;
        self.contexts.push(ctx);
    }

    pub(super) fn disconnect(&mut self, device_id: u32) {
        self.contexts.retain(|ctx| ctx.device_id != device_id);
    }

    pub fn device_ids(&self) -> Vec<u32> {
        self.contexts.iter().map(|ctx| ctx.device_id).collect()
    }

    fn ctx_mut(&mut self, id: u32) -> Result<&mut Context> {
        self.contexts
            .iter_mut()
            .find(|ctx| ctx.device_id == id)
            .ok_or(LibUsbError::NotFound.into())
    }

    fn new() -> Self {
        Self {
            contexts: Vec::new(),
            next_id: 0,
        }
    }
}

/// Manage context of each device.
struct Context {
    device: Device,
    device_id: u32,
    channel: Arc<Mutex<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)>>,

    /// Hold interface state.
    /// Currently just holds claimed state.
    iface_state: HashMap<IfaceKind, bool>,
}

impl Context {
    fn run(mut device: Device, device_id: u32) -> Self {
        let channel = device.run();
        let iface_state = vec![
            (IfaceKind::Control, false),
            (IfaceKind::Event, false),
            (IfaceKind::Stream, false),
        ]
        .into_iter()
        .collect();

        Self {
            device,
            device_id,
            channel: Arc::new(Mutex::new(channel)),
            iface_state,
        }
    }

    fn claim_interface(
        &mut self,
        iface: IfaceKind,
    ) -> Result<Arc<Mutex<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)>>> {
        if self.is_claimed(iface) {
            Err(LibUsbError::Busy.into())
        } else {
            *self.iface_state.get_mut(&iface).unwrap() = true;
            Ok(self.channel.clone())
        }
    }

    fn release_interface(&mut self, iface: IfaceKind) {
        *self.iface_state.get_mut(&iface).unwrap() = false;
    }

    fn is_claimed(&self, iface: IfaceKind) -> bool {
        self.iface_state[&iface]
    }

    fn shutdown(&mut self) {
        self.device.shutdown()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.device.shutdown()
    }
}
