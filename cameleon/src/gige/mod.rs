/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod control_handle;
pub mod register_map;
pub mod stream_handle;

pub use control_handle::ControlHandle;
pub use stream_handle::StreamHandle;

use std::time;

use cameleon_device::gige;

use crate::ControlError;

use async_std::task;

use super::{CameleonResult, Camera, CameraInfo};

const ENUMERATION_TIMEOUT: time::Duration = time::Duration::from_millis(500);

impl From<gige::Error> for ControlError {
    fn from(err: gige::Error) -> Self {
        match err {
            gige::Error::Io(err) => ControlError::Io(err.into()),
            gige::Error::InvalidPacket(msg) => ControlError::InvalidData(msg.into()),
            gige::Error::InvalidData(msg) => ControlError::InvalidData(msg.into()),
        }
    }
}

pub fn enumerate_cameras() -> CameleonResult<Vec<Camera<ControlHandle, StreamHandle>>> {
    let device_infos =
        task::block_on(gige::enumerate_devices(ENUMERATION_TIMEOUT)).map_err(ControlError::from)?;

    let mut cameras: Vec<Camera<ControlHandle, StreamHandle>> =
        Vec::with_capacity(device_infos.len());
    for info in device_infos {
        let camera_info = CameraInfo {
            vendor_name: info.manufacturer_name.clone(),
            model_name: info.model_name.clone(),
            serial_number: info.serial_number.clone(),
        };
        let ctrl_handle = unwrap_or_log!(ControlHandle::new(info));
        let strm_handle = StreamHandle {};
        cameras.push(Camera::new(ctrl_handle, strm_handle, None, camera_info));
    }

    Ok(cameras)
}
