use std::{collections::HashMap, sync::Arc};

use async_std::{
    sync::{Mutex, Receiver, Sender},
    task,
};
use lazy_static::lazy_static;

use crate::usb3::{DeviceInfo, LibUsbError, Result};

use super::{
    device::Device,
    fake_protocol::{FakeAckPacket, FakeReqPacket, IfaceKind},
};

pub(super) type DevicePipe = Arc<Mutex<(Sender<FakeReqPacket>, Receiver<FakeAckPacket>)>>;

lazy_static! {
    static ref DEVICE_POOL: Arc<Mutex<DevicePool>> = Arc::new(Mutex::new(DevicePool::new()));
}

pub(crate) struct DevicePool {
    contexts: Vec<Context>,
    next_id: u32,
}

impl DevicePool {
    pub(crate) fn device_info(&self, device_id: u32) -> Result<DeviceInfo> {
        let ctx = self.ctx(device_id)?;
        Ok(ctx.device_info())
    }

    pub(crate) fn device_ids(&self) -> Vec<u32> {
        self.contexts.iter().map(|ctx| ctx.device_id).collect()
    }

    pub(crate) fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut DevicePool) -> R,
    {
        let mut pool = task::block_on(DEVICE_POOL.lock());
        f(&mut *pool)
    }

    pub(super) fn claim_interface(
        &mut self,
        device_id: u32,
        iface: IfaceKind,
    ) -> Result<DevicePipe> {
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

    fn ctx_mut(&mut self, id: u32) -> Result<&mut Context> {
        self.contexts
            .iter_mut()
            .find(|ctx| ctx.device_id == id)
            .ok_or_else(|| LibUsbError::NotFound.into())
    }

    fn ctx(&self, id: u32) -> Result<&Context> {
        self.contexts
            .iter()
            .find(|ctx| ctx.device_id == id)
            .ok_or_else(|| LibUsbError::NotFound.into())
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

    fn claim_interface(&mut self, iface: IfaceKind) -> Result<DevicePipe> {
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

    fn device_info(&self) -> DeviceInfo {
        self.device.device_info().clone()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.device.shutdown()
    }
}
