/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Condvar, Mutex};

use futures_channel::oneshot;

use crate::{payload::PayloadSender, DeviceControl, PayloadStream, StreamError, StreamResult};

use super::register_map::{Bootstrap, StreamRegister};

pub struct StreamHandle {
    buf_size: usize,
    completion: Option<Arc<(Mutex<bool>, Condvar)>>,
    cancellation_tx: Option<oneshot::Sender<()>>,
}

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
        ctrl: &mut dyn DeviceControl,
    ) -> StreamResult<()> {
        if self.is_loop_running() {
            return Err(StreamError::InStreaming);
        }
        let (cancellation_tx, cancellation_rx) = oneshot::channel();
        self.cancellation_tx = Some(cancellation_tx);
        let completion = Arc::new((Mutex::new(false), Condvar::new()));
        self.completion = Some(completion.clone());
        StreamLoop {
            buffer: vec![0u8; self.buf_size],
            cancellation_rx,
            completion,
        }
        .run();
        Ok(())
    }

    fn stop_streaming_loop(&mut self) -> StreamResult<()> {
        todo!();
    }

    fn is_loop_running(&self) -> bool {
        if let Some((completed, _)) = self.completion.as_ref().map(|v| v.as_ref()) {
            if !*completed.lock().unwrap() {
                return true;
            }
        }
        false
    }
}

struct StreamLoop {
    buffer: Vec<u8>,
    cancellation_rx: oneshot::Receiver<()>,
    completion: Arc<(Mutex<bool>, Condvar)>,
}

impl StreamLoop {
    fn run(mut self) {}
}
