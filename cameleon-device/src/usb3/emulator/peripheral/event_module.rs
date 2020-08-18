use async_std::{
    prelude::*,
    sync::{Receiver, Sender},
};

use futures::channel::oneshot;

use super::signal::EventSignal;

pub(super) struct EventModule {
    ctrl_rx: Receiver<EventSignal>,
    ack_tx: Sender<Vec<u8>>,
    timestamp: u64,
    enabled: bool,
}

impl EventModule {
    pub(super) fn new(ctrl_rx: Receiver<EventSignal>, ack_tx: Sender<Vec<u8>>, timestamp: u64) -> Self {
        Self {
            ctrl_rx,
            ack_tx,
            timestamp,
            enabled: false
        }
    }


    pub(super) async fn run(mut self, _completed: oneshot::Sender<()>) {
        while let Some(signal) = self.ctrl_rx.next().await {
            match signal {
                EventSignal::EventData{event_id, data, request_id} => {
                    if self.enabled {
                        self.send_event_data( event_id, &data, request_id)
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

    fn send_event_data(&self,  event_id: u16, data: &[u8], request_id: u16) {
        let scd = match event_packet::EventScd::single_event(event_id, data, self.timestamp) {
            Ok(scd) => scd,
            Err(e) => {
                log::error!("can't generate event packet: cause {}", e);
                return;
            }
        };

        let mut buf = vec![];
        if let Err(e) = scd.finalize(request_id).serialize(&mut buf) {
            log::error!("cant't serialize event packet: cause {}", e);
            return;
        }

        if let Err(e) = self.ack_tx.try_send(buf) {
            log::error!("can't send event packet to interface of the device: cause {}", e);
        }
    }
}

mod event_packet {
    use std::borrow::Cow;
    use std::convert::TryInto;
    use std::io::Write;

    use thiserror::Error;
    use byteorder::{LE, WriteBytesExt};

    pub(super) use crate::usb3::protocol::event::EventScd;


    #[derive(Debug, Error)]
    pub(super) enum ProtocolError {
        #[error("packet is broken: {}", 0)]
        InvalidPacket(Cow<'static, str>),

        #[error("internal buffer for a packet is something wrong")]
        BufferError(#[from] std::io::Error),
    }

    pub(super) type ProtocolResult<T> = std::result::Result<T, ProtocolError>;


    pub(super) struct EventPacket<'a> {
        scd_len: u16,
        request_id: u16,
        scd: EventScd<'a>
    }

    impl<'a> EventPacket<'a> {
        const PREFIX_MAGIC: u32 = 0x45563355;
        const COMMAND_FLAG: u16 = 0b1 << 14;
        const COMMAND_ID: u16 = 0x0C00;


        fn from_scd(scd: EventScd<'a>, request_id: u16) -> Self {
            Self {
                scd_len: scd.scd_len_unchecked(),
                request_id,
                scd,
            }
        }

        pub(super) fn serialize(&self, mut buf: impl Write) -> ProtocolResult<()>{
            // Serialize CCD.
            buf.write_u32::<LE>(Self::PREFIX_MAGIC)?;
            buf.write_u16::<LE>(Self::COMMAND_FLAG)?;
            buf.write_u16::<LE>(Self::COMMAND_ID)?;
            buf.write_u16::<LE>(self.scd.scd_len_unchecked())?;
            buf.write_u16::<LE>(self.request_id)?;

            // Serialize SCD.
            self.scd.serialize(buf)?;
            Ok(())
        }

    }

    // TODO: Implement Multievent.
    impl<'a> EventScd<'a> {
        pub(super) fn single_event(event_id: u16, data: &'a [u8], timestamp: u64) -> ProtocolResult<Self> {
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
            buf.write_u16::<LE>(self.event_size)?;
            buf.write_u16::<LE>(self.event_id)?;
            buf.write_u64::<LE>(self.timestamp)?;
            buf.write_all(self.data)?;
            Ok(())
        }

        fn scd_len_unchecked(&self) -> u16 {
            self.scd_len_checked().unwrap()
        }

        fn scd_len_checked(&self) -> ProtocolResult<u16> {
            // event_size(2bytes) + event_id(2bytes) + timestamp(8bytes) + data_len
            let data_len: u16 = self.data.len().try_into().map_err(|_| ProtocolError::InvalidPacket("event data size is larger than u16::MAX".into()))?;
            (2u16 + 2u16 + 8u16).checked_add(data_len).ok_or(ProtocolError::InvalidPacket("scd size is larger than u16::MAX".into()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::usb3::protocol::event;

        #[test]
        fn test_single_event() {
            let data = &[1, 2, 3];
            let timestamp = 123456789;
            let event_id = 0xff;
            let event_pacekt = EventScd::single_event(event_id, data, timestamp).unwrap().finalize(10);
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

    use async_std::{sync::{channel, TryRecvError}, task, future::timeout};
    use futures::channel::oneshot;

    use crate::usb3::protocol::event;

    use super::*;

    const TO: Duration = Duration::from_millis(100);

    fn new_module() -> (EventModule, Sender<EventSignal>, Receiver<Vec<u8>>) {
        let (ctrl_tx, ctrl_rx) = channel(10);
        let (ack_tx, ack_rx) = channel(10);
        (EventModule::new(ctrl_rx, ack_tx, 0), ctrl_tx, ack_rx)
    }

    #[test]
    fn test_run_and_stop() {
        let (event_module, ctrl_tx, ack_rx) = new_module();
        let (completed_tx, completed_rx) = oneshot::channel();
        task::spawn(event_module.run(completed_tx));

        assert!(ctrl_tx.try_send(EventSignal::Shutdown).is_ok());
        let completed = task::block_on(timeout(TO, completed_rx)).unwrap();
        assert_eq!(completed, Err(oneshot::Canceled));

        assert_eq!(ack_rx.try_recv(), Err(TryRecvError::Disconnected));
    }

    #[test]
    fn test_signal() {
        let (event_module, ctrl_tx, ack_rx) = new_module();
        let (completed_tx, completed_rx) = oneshot::channel();
        task::spawn(event_module.run(completed_tx));
        assert!(ctrl_tx.try_send(EventSignal::Enable).is_ok());

        // Test EventData signal.
        let event_id = 10;
        let request_id = 20;
        let data = vec![1, 2, 3];
        ctrl_tx.try_send(EventSignal::EventData{event_id, data: data.clone(), request_id}).unwrap();
        let received = task::block_on(timeout(TO, ack_rx.recv())).unwrap().unwrap();
        let event_packet = event::EventPacket::parse(&received).unwrap();
        assert_eq!(event_packet.request_id(), request_id);
        assert_eq!(event_packet.scd.len(), 1);
        assert_eq!(&event_packet.scd[0].data, &data.as_slice());

        // Test UpdateTimestamp signal.
        let timestamp = 123456789;
        ctrl_tx.try_send(EventSignal::UpdateTimestamp(timestamp)).unwrap();
        ctrl_tx.try_send(EventSignal::EventData{event_id, data: data.clone(), request_id}).unwrap();
        let received = task::block_on(timeout(TO, ack_rx.recv())).unwrap().unwrap();
        let event_packet = event::EventPacket::parse(&received).unwrap();
        assert_eq!(event_packet.scd[0].timestamp, timestamp);

        // Clean up.
        assert!(ctrl_tx.try_send(EventSignal::Shutdown).is_ok());
        let completed = task::block_on(timeout(TO, completed_rx)).unwrap();
        assert_eq!(completed, Err(oneshot::Canceled));
        assert_eq!(ack_rx.try_recv(), Err(TryRecvError::Disconnected));

    }

}
