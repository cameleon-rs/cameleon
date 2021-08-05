/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{payload::PayloadSender, DeviceControl, PayloadStream, StreamResult};

pub struct StreamHandle {}

impl PayloadStream for StreamHandle {
    fn open(&mut self) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn close(&mut self) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn start_streaming_loop(
        &mut self,
        _sender: PayloadSender,
        _ctrl: &mut dyn DeviceControl,
    ) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn stop_streaming_loop(&mut self) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn is_loop_running(&self) -> bool {
        // TODO:
        false
    }
}
