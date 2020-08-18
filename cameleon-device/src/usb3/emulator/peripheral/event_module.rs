use async_std::{
    prelude::*,
    sync::{Receiver, Sender},
};

use futures::channel::oneshot;

use super::signal::EventSignal;

pub(super) struct EventModule {
    ctrl_rx: Receiver<EventSignal>,
    ack_tx: Sender<Vec<u8>>,
    timestamp: u32,
    enabled: bool,
}

impl EventModule {
    pub(super) async fn run(mut self, _completed: oneshot::Sender<()>) {
        while let Some(signal) = self.ctrl_rx.next().await {
            match signal {
                EventSignal::EventData(event_id) => {
                    if self.enabled {
                        todo!();
                    } else {
                        log::warn! {"receive event data signal, but event module is currently disabled"}
                    }
                }
                EventSignal::UpdateTimestamp(timestamp) => {
                    self.timestamp = timestamp;
                }
                EventSignal::Enable => {
                    if self.enabled {
                        log::warn! {"receive event enable signal, but event module is already enabled"}
                    } else {
                        self.enabled = true;
                        log::info! {"event module is enabled"};
                    }
                }
                EventSignal::Disable(_completed) => {
                    if self.enabled {
                        self.enabled = false;
                        log::info! {"event module is disenabled"};
                    } else {
                        log::warn! {"receive event disable signal, but event module is already disabled"}
                    }
                }
                EventSignal::Shutdown => break,
            }
        }
    }
}
