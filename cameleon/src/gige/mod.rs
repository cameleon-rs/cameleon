/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod control_handle;
pub mod register_map;

use cameleon_device::gige;

use crate::ControlError;

impl From<gige::Error> for ControlError {
    fn from(err: gige::Error) -> Self {
        match err {
            gige::Error::Io(err) => ControlError::Io(err.into()),
            gige::Error::InvalidPacket(msg) => ControlError::InvalidData(msg.into()),
            gige::Error::InvalidData(msg) => ControlError::InvalidData(msg.into()),
        }
    }
}
