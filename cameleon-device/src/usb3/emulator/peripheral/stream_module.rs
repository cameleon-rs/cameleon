use async_std::{
    prelude::*,
    sync::{Receiver, Sender},
};
use futures::channel::oneshot;

use super::signal::StreamSignal;

// TODO: Implement stream module.
pub(super) struct StreamModule {
    ctrl_rx: Receiver<StreamSignal>,
    ack_tx: Sender<Vec<u8>>,
    enabled: bool,
}

impl StreamModule {
    pub(super) fn new(ctrl_rx: Receiver<StreamSignal>, ack_tx: Sender<Vec<u8>>) -> Self {
        Self {
            ctrl_rx,
            ack_tx,
            enabled: false,
        }
    }

    pub(super) async fn run(mut self, _completed: oneshot::Sender<()>) {
        while let Some(signal) = self.ctrl_rx.next().await {
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
                StreamSignal::Shutdown => break,
            }
        }
    }
}
