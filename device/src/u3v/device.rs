/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::u3v::{DeviceInfo, Result};

use super::channel::{ControlChannel, ControlIfaceInfo, ReceiveChannel, ReceiveIfaceInfo};

pub(super) type RusbDevHandle = rusb::DeviceHandle<rusb::GlobalContext>;
pub(super) type RusbDevice = rusb::Device<rusb::GlobalContext>;

/// Entry point to the connected device.
/// This device itself doesn't communicate with the connected device but provide basic device
/// information and channels to communicate with the connected device. So it's valid to use
/// provided channels even after dropping this instance.
#[derive(Debug)]
pub struct Device {
    device: RusbDevice,

    ctrl_iface_info: ControlIfaceInfo,
    event_iface_info: Option<ReceiveIfaceInfo>,
    stream_iface_info: Option<ReceiveIfaceInfo>,

    pub device_info: DeviceInfo,
}

impl Device {
    pub fn control_channel(&self) -> Result<ControlChannel> {
        let device_handle = self.device.open()?;

        Ok(ControlChannel::new(
            device_handle,
            self.ctrl_iface_info.clone(),
        ))
    }

    pub fn event_channel(&self) -> Result<Option<ReceiveChannel>> {
        match &self.event_iface_info {
            Some(iface_info) => {
                let device_handle = self.device.open()?;
                Ok(Some(ReceiveChannel::new(device_handle, iface_info.clone())))
            }
            None => Ok(None),
        }
    }

    pub fn stream_channel(&self) -> Result<Option<ReceiveChannel>> {
        match &self.stream_iface_info {
            Some(iface_info) => {
                let device_handle = self.device.open()?;
                Ok(Some(ReceiveChannel::new(device_handle, iface_info.clone())))
            }
            None => Ok(None),
        }
    }

    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    pub(super) fn new(
        device: RusbDevice,
        ctrl_iface_info: ControlIfaceInfo,
        event_iface_info: Option<ReceiveIfaceInfo>,
        stream_iface_info: Option<ReceiveIfaceInfo>,
        device_info: DeviceInfo,
    ) -> Self {
        let device = Self {
            device,
            ctrl_iface_info,
            event_iface_info,
            stream_iface_info,
            device_info,
        };

        log::info! {"{}: create device", device.log_name()};

        device
    }

    //TODO: We need logger.
    fn log_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.device_info.vendor_name,
            self.device_info.model_name,
            self.device_info.serial_number,
        )
    }
}
