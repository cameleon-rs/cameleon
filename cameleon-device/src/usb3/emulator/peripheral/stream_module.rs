use async_std::{
    prelude::*,
    sync::{Receiver, Sender},
};

use super::{device::Timestamp, signal::StreamSignal};

// TODO: Implement stream module.
pub(super) struct StreamModule {
    enabled: bool,
    timestamp: Timestamp,
}

impl StreamModule {
    pub(super) fn new(timestamp: Timestamp) -> Self {
        Self {
            enabled: false,
            timestamp,
        }
    }

    pub(super) async fn run(
        mut self,
        mut ctrl_rx: Receiver<StreamSignal>,
        ack_tx: Sender<Vec<u8>>,
    ) {
        let mut completed = None;

        while let Some(signal) = ctrl_rx.next().await {
            match signal {
                StreamSignal::Enable => {
                    if self.enabled {
                        log::warn! {"receive event enable signal, but event module is already enabled"}
                    } else {
                        self.enabled = true;
                        log::info! {"event module is enabled"};
                    }
                }
                StreamSignal::Disable(_completed) => {
                    if self.enabled {
                        self.enabled = false;
                        log::info! {"event module is disenabled"};
                    } else {
                        log::warn! {"receive event disable signal, but event module is already disabled"}
                    }
                }
                StreamSignal::Shutdown(completed_tx) => {
                    completed = Some(completed_tx);
                    break;
                }
            }
        }

        if completed.is_none() {
            log::error!("stream module ends abnormally. cause: stream signal sender is dropped");
        }
    }
}
