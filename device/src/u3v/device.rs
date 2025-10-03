/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nusb::Device as NusbDevice;

use crate::u3v::{DeviceInfo, Result};

use super::channel::{ControlChannel, ControlIfaceInfo, ReceiveChannel, ReceiveIfaceInfo};

/// Entry point to the connected device.
/// This device itself doesn't communicate with the connected device but provides basic device
/// information and channels to communicate with the connected device. So it's valid to use
/// provided channels even after dropping this instance.
#[derive(Clone)]
pub struct Device {
    device: NusbDevice,

    ctrl_iface_info: ControlIfaceInfo,
    event_iface_info: Option<ReceiveIfaceInfo>,
    stream_iface_info: Option<ReceiveIfaceInfo>,

    pub device_info: DeviceInfo,
}

impl Device {
    pub fn control_channel(&self) -> Result<ControlChannel> {
        ControlChannel::new(self.device.clone(), self.ctrl_iface_info.clone())
    }

    pub fn event_channel(&self) -> Result<Option<ReceiveChannel>> {
        match &self.event_iface_info {
            Some(iface_info) => {
                ReceiveChannel::new(self.device.clone(), iface_info.clone()).map(Some)
            }
            None => Ok(None),
        }
    }

    pub fn stream_channel(&self) -> Result<Option<ReceiveChannel>> {
        match &self.stream_iface_info {
            Some(iface_info) => {
                ReceiveChannel::new(self.device.clone(), iface_info.clone()).map(Some)
            }
            None => Ok(None),
        }
    }

    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    pub(super) fn new(
        device: NusbDevice,
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

        log::info!("{}: create device", device.log_name());
        device
    }

    fn log_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.device_info.vendor_name,
            self.device_info.model_name,
            self.device_info.serial_number,
        )
    }
}
