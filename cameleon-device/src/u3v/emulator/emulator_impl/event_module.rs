use async_std::{
    channel::{Receiver, Sender},
    prelude::*,
};

use super::{
    shared_queue::SharedQueue,
    signal::{EventSignal, InterfaceSignal},
    IfaceKind,
};

pub(super) struct EventModule {
    queue: SharedQueue<Vec<u8>>,
    timestamp: u64,

    enabled: bool,
}

impl EventModule {
    pub(super) fn new(queue: SharedQueue<Vec<u8>>, timestamp: u64) -> Self {
        Self {
            queue,
            timestamp,
            enabled: false,
        }
    }

    pub(super) async fn run(
        mut self,
        signal_tx: Sender<InterfaceSignal>,
        mut signal_rx: Receiver<EventSignal>,
    ) {
        while let Some(signal) = signal_rx.next().await {
            match signal {
                EventSignal::_EventData {
                    event_id,
                    data,
                    request_id,
                } => {
                    if self.enabled {
                        self.enqueue_or_halt(event_id, &data, request_id, &signal_tx)
                    } else {
                        log::warn! {"receive event data signal, but event module is currently disabled"}
                    }
                }

                EventSignal::UpdateTimestamp(timestamp) => {
                    self.timestamp = timestamp;
                }

                EventSignal::_Enable => {
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
                        log::info! {"event module is disabled"};
                    } else {
                        log::warn! {"receive event disable signal, but event module is already disabled"}
                    }
                }

                EventSignal::Shutdown => {
                    break;
                }
            }
        }
    }

    fn enqueue_or_halt(
        &mut self,
        event_id: u16,
        data: &[u8],
        request_id: u16,
        signal_tx: &Sender<InterfaceSignal>,
    ) {
        let scd = match event_packet::EventScd::single_event(event_id, data, self.timestamp) {
            Ok(scd) => scd,
            Err(e) => {
                log::error!("can't generate event packet: cause {}", e);
                return;
            }
        };

        let mut bytes = vec![];
        if let Err(e) = scd.finalize(request_id).serialize(&mut bytes) {
            log::error!("cant't serialize event packet: cause {}", e);
            return;
        }

        if !self.queue.enqueue(bytes) {
            log::warn!("event queue is full, entering a halted state",);

            let signal = InterfaceSignal::Halt(IfaceKind::Event);

            match signal_tx.try_send(signal) {
                Ok(()) => {}
                Err(_) => {
                    log::error!("Control module -> Interface channel is full");
                }
            }
        }
    }
}

mod event_packet {
    pub(super) use crate::u3v::protocol::event::EventScd;

    use std::borrow::Cow;
    use std::convert::TryInto;
    use std::io::Write;

    use thiserror::Error;

    use crate::u3v::protocol::util::WriteBytes;


    #[derive(Debug, Error)]
    pub(super) enum ProtocolError {
        #[error("packet is broken: {}", 0)]
        InvalidPacket(Cow<'static, str>),

        #[error("internal buffer for a packet is something wrong")]
        BufferError(#[from] std::io::Error),
    }

    pub(super) type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

    pub(super) struct EventPacket<'a> {
        _scd_len: u16,
        request_id: u16,
        scd: EventScd<'a>,
    }

    impl<'a> EventPacket<'a> {
        const PREFIX_MAGIC: u32 = 0x45563355;
        const COMMAND_FLAG: u16 = 0b1 << 14;
        const COMMAND_ID: u16 = 0x0C00;

        fn from_scd(scd: EventScd<'a>, request_id: u16) -> Self {
            Self {
                _scd_len: scd.scd_len_unchecked(),
                request_id,
                scd,
            }
        }

        pub(super) fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            // Serialize CCD.
            buf.write_bytes(Self::PREFIX_MAGIC)?;
            buf.write_bytes(Self::COMMAND_FLAG)?;
            buf.write_bytes(Self::COMMAND_ID)?;
            buf.write_bytes(self.scd.scd_len_unchecked())?;
            buf.write_bytes(self.request_id)?;

            // Serialize SCD.
            self.scd.serialize(buf)?;
            Ok(())
        }
    }

    // TODO: Implement Multievent.
    impl<'a> EventScd<'a> {
        pub(super) fn single_event(
            event_id: u16,
            data: &'a [u8],
            timestamp: u64,
        ) -> ProtocolResult<Self> {
            let scd = EventScd {
                event_size: 0,
                event_id,
                timestamp,
                data,
            };
            scd.scd_len_checked()?;
            Ok(scd)
        }

        pub(super) fn finalize(self, request_id: u16) -> EventPacket<'a> {
            EventPacket::from_scd(self, request_id)
        }

        fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()> {
            buf.write_bytes(self.event_size)?;
            buf.write_bytes(self.event_id)?;
            buf.write_bytes(self.timestamp)?;
            buf.write_all(self.data)?;
            Ok(())
        }

        fn scd_len_unchecked(&self) -> u16 {
            self.scd_len_checked().unwrap()
        }

        fn scd_len_checked(&self) -> ProtocolResult<u16> {
            // event_size(2bytes) + event_id(2bytes) + timestamp(8bytes) + data_len
            let data_len: u16 = self.data.len().try_into().map_err(|_| {
                ProtocolError::InvalidPacket("event data size is larger than u16::MAX".into())
            })?;
            (2u16 + 2u16 + 8u16).checked_add(data_len).ok_or_else(|| {
                ProtocolError::InvalidPacket("scd size is larger than u16::MAX".into())
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::u3v::protocol::event;

        #[test]
        fn test_single_event() {
            let data = &[1, 2, 3];
            let timestamp = 123456789;
            let event_id = 0xff;
            let event_pacekt = EventScd::single_event(event_id, data, timestamp)
                .unwrap()
                .finalize(10);
            let mut buf = vec![];
            event_pacekt.serialize(&mut buf).unwrap();

            let parsed = event::EventPacket::parse(&buf).unwrap();

            assert_eq!(parsed.request_id(), 10);
            assert_eq!(parsed.scd.len(), 1);
            assert_eq!(parsed.scd[0].event_id, event_id);
            assert_eq!(parsed.scd[0].timestamp, timestamp);
            assert_eq!(parsed.scd[0].data, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_std::{channel, future::timeout, task};

    use crate::u3v::protocol::event;

    use super::*;

    const TO: Duration = Duration::from_millis(100);

    fn spawn_module() -> (
        Sender<EventSignal>,
        Receiver<InterfaceSignal>,
        SharedQueue<Vec<u8>>,
    ) {
        let (signal_tx, signal_rx) = channel::bounded(10);
        let (iface_signal_tx, iface_signal_rx) = channel::bounded(10);
        let queue = SharedQueue::new(10);
        let event_module = EventModule::new(queue.clone(), 0);
        task::spawn(event_module.run(iface_signal_tx, signal_rx));

        (signal_tx, iface_signal_rx, queue)
    }

    fn receive_data(queue: &SharedQueue<Vec<u8>>) -> Option<Vec<u8>> {
        let now = std::time::Instant::now();
        while now.elapsed() < TO {
            if let Some(data) = queue.dequeue() {
                return Some(data);
            }
        }

        None
    }

    #[test]
    fn test_run_and_stop() {
        let (signal_tx, mut iface_signal_rx, _) = spawn_module();

        assert!(signal_tx.try_send(EventSignal::Shutdown).is_ok());
        task::block_on(timeout(TO, iface_signal_rx.next())).unwrap();
    }

    #[test]
    fn test_signal() {
        let (signal_tx, mut iface_signal_rx, queue) = spawn_module();
        signal_tx.try_send(EventSignal::_Enable).unwrap();

        // Test EventData signal.
        let event_id = 10;
        let request_id = 20;
        let data = vec![1, 2, 3];
        signal_tx
            .try_send(EventSignal::_EventData {
                event_id,
                data: data.clone(),
                request_id,
            })
            .unwrap();

        let received = receive_data(&queue).unwrap();

        let event_packet = event::EventPacket::parse(&received).unwrap();
        assert_eq!(event_packet.request_id(), request_id);
        assert_eq!(event_packet.scd.len(), 1);
        assert_eq!(&event_packet.scd[0].data, &data.as_slice());

        // Test UpdateTimestamp signal.
        let timestamp = 123456789;
        signal_tx
            .try_send(EventSignal::UpdateTimestamp(timestamp))
            .unwrap();
        signal_tx
            .try_send(EventSignal::_EventData {
                event_id,
                data,
                request_id,
            })
            .unwrap();
        let received = receive_data(&queue).unwrap();
        let event_packet = event::EventPacket::parse(&received).unwrap();
        assert_eq!(event_packet.scd[0].timestamp, timestamp);

        // Clean up.
        assert!(signal_tx.try_send(EventSignal::Shutdown).is_ok());
        task::block_on(timeout(TO, iface_signal_rx.next())).unwrap();
    }
}
