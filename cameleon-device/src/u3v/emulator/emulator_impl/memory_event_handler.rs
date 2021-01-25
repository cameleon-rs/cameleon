use async_std::channel::{Receiver, Sender, self};

use cameleon_impl::memory::{prelude::*, MemoryObserver};

use super::{control_module::Worker, control_protocol::*, memory::{ABRM, Memory}, signal::*};


const MEMORY_EVENT_CHANNEL_CAPACITY: usize = 100;

#[derive(Clone)]
pub(super) struct MemoryEventHandler {
    rx: Receiver<MemoryEvent>,
}

impl MemoryEventHandler {
    /// Contruct MemoryEventHandler while registering observer to memory.
    pub(super) async fn new(memory: &mut Memory) -> Self{
        let (tx, rx) = channel::bounded(MEMORY_EVENT_CHANNEL_CAPACITY);
        memory.register_observer::<ABRM::TimestampLatch, _>(TimestampLatchHandler { sender: tx });

        MemoryEventHandler { rx }
    }

    /// Handle write events, return Some(error_ack) if an error occurs while handling write events.
    pub(super) async fn handle_events(
        &self,
        worker: &Worker,
        scd_kind: cmd::ScdKind,
    ) -> Option<ack::ErrorAck> {
        let mut error_ack = None;

        while let Ok(event) = self.rx.try_recv() {
            let ack = match event {
                MemoryEvent::TimestampLatch => {
                    TimestampLatchHandler::handle_events(worker, scd_kind).await
                }
            };
            error_ack = error_ack.or(ack);
        }
        error_ack
    }
}

/// Handle events caused by writes to `TiemStampLatch` regsiter.
///
/// If 1 is written to `TiemStampLatch`, `TimeStamp` register must be updated with current device time stamp.
struct TimestampLatchHandler {
    sender: Sender<MemoryEvent>,
}

impl TimestampLatchHandler {
    async fn handle_events(worker: &Worker, scd_kind: cmd::ScdKind) -> Option<ack::ErrorAck> {
        let mut memory = worker.memory.lock().await;
        match memory.read::<ABRM::TimestampLatch>() {
            Ok(value) => {
                // Write any number other than 1 cause error.
                if value != 1 {
                    return Some(
                        ack::ErrorAck::new(ack::GenCpStatus::GenericError, scd_kind).into(),
                    );
                }
            }
            Err(e) => {
                log::warn!("failed to read ABRM::TimestampLatch {}", e);
                return Some(ack::ErrorAck::new(ack::GenCpStatus::GenericError, scd_kind).into());
            }
        }

        // Write current time stamp to `TimeStamp` register.
        let timestamp_ns = worker.timestamp.as_nanos().await;
        if let Err(e) = memory.write::<ABRM::Timestamp>(timestamp_ns) {
            log::warn!("failed to write to ABRM::Timestamp register {}", e)
        }

        // Send signal to [`super::event_module::EventModule`] to notify `TimeStamp` register is updated.
        let signal = EventSignal::UpdateTimestamp(timestamp_ns);
        worker.try_send_signal(signal);

        None
    }
}

impl MemoryObserver for TimestampLatchHandler {
    fn update(&self) {
        if let Err(e) = self.sender.try_send(MemoryEvent::TimestampLatch) {
            log::warn!("memory observer error: {}", e);
        }
    }
}

enum MemoryEvent {
    TimestampLatch,
}
