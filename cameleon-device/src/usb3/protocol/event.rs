use std::io::Cursor;

use byteorder::{ReadBytesExt, LE};

use crate::usb3::{Error, Result};

use super::parse_util;

pub struct EventPacket<'a> {
    ccd: EventCcd,
    pub scd: Vec<EventScd<'a>>,
}

impl<'a> EventPacket<'a> {
    const PREFIX_MAGIC: u32 = 0x45563355;

    pub fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> Result<Self> {
        let mut cursor = Cursor::new(buf.as_ref());

        Self::parse_prefix(&mut cursor)?;

        let ccd = EventCcd::parse(&mut cursor)?;

        let scd = EventScd::parse(&mut cursor, &ccd)?;

        Ok(Self { ccd, scd })
    }

    pub fn request_id(&self) -> u16 {
        self.ccd.request_id
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let magic = cursor.read_u32::<LE>()?;
        if magic == Self::PREFIX_MAGIC {
            Ok(())
        } else {
            Err(Error::InvalidPacket("invalid event prefix magic".into()))
        }
    }
}

struct EventCcd {
    #[allow(unused)]
    pub(crate) flag: u16,
    #[allow(unused)]
    pub(crate) command_id: u16,
    pub(crate) scd_len: u16,
    pub(crate) request_id: u16,
}

impl EventCcd {
    const EVENT_COMMAND_ID: u16 = 0x0c00;

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let flag = cursor.read_u16::<LE>()?;
        let command_id = cursor.read_u16::<LE>()?;
        if command_id != Self::EVENT_COMMAND_ID {
            return Err(Error::InvalidPacket("invalid event command id".into()));
        }
        let scd_len = cursor.read_u16::<LE>()?;
        let request_id = cursor.read_u16::<LE>()?;
        Ok({
            Self {
                flag,
                command_id,
                scd_len,
                request_id,
            }
        })
    }
}

pub struct EventScd<'a> {
    #[allow(unused)]
    pub event_size: u16,
    pub event_id: u16,
    pub timestamp: u64,
    pub data: &'a [u8],
}

impl<'a> EventScd<'a> {
    fn parse(cursor: &mut Cursor<&'a [u8]>, ccd: &EventCcd) -> Result<Vec<Self>> {
        let mut events = vec![];
        let mut remained = ccd.scd_len;

        while remained > 0 {
            let event_size = cursor.read_u16::<LE>()?;
            let event_id = cursor.read_u16::<LE>()?;
            let timestamp = cursor.read_u64::<LE>()?;

            // MultiEvent isn't enabled.
            let data = if event_size == 0 {
                remained = remained.checked_sub(12).ok_or_else(|| {
                    Error::InvalidPacket("SCD length in CCD is inconsistent with SCD".into())
                })?;
                let data = parse_util::read_bytes(cursor, remained)?;
                remained = 0;
                data
            } else {
                let data_len = event_size.checked_sub(12).ok_or_else(|| {
                    Error::InvalidPacket("event size is smaller than scd header".into())
                })?;
                remained = remained.checked_sub(event_size).ok_or_else(|| {
                    Error::InvalidPacket("SCD length in CCD is inconsistent with SCD".into())
                })?;
                parse_util::read_bytes(cursor, data_len)?
            };

            events.push(EventScd {
                event_size,
                event_id,
                timestamp,
                data,
            });
        }

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use byteorder::WriteBytesExt;

    use super::*;

    fn serialize_header(scd_len: u16, request_id: u16) -> Vec<u8> {
        let mut ccd = vec![];
        ccd.write_u32::<LE>(0x45563355).unwrap(); // Magic.
        ccd.write_u16::<LE>(1 << 14).unwrap(); // Request ack for now.
        ccd.write_u16::<LE>(0x0c00).unwrap();
        ccd.write_u16::<LE>(scd_len).unwrap();
        ccd.write_u16::<LE>(request_id).unwrap();
        ccd
    }

    #[test]
    fn test_single_event() {
        let mut scd = vec![];
        scd.write_u16::<LE>(0).unwrap(); // Single event.
        scd.write_u16::<LE>(0x10).unwrap(); // Dummy event ID.
        let timestamp = 0x123456789abcdef;
        scd.write_u64::<LE>(timestamp).unwrap();
        let data = &[0x12, 0x34];
        scd.extend(data);

        let mut raw_packet = serialize_header(scd.len() as u16, 1);
        raw_packet.extend(scd);

        let event_packet = EventPacket::parse(&raw_packet).unwrap();

        assert_eq!(event_packet.request_id(), 1);
        assert_eq!(event_packet.scd.len(), 1);
        assert_eq!(event_packet.scd[0].event_id, 0x10);
        assert_eq!(event_packet.scd[0].timestamp, 0x123456789abcdef);
        assert_eq!(event_packet.scd[0].data, data);
    }

    #[test]
    fn test_multi_event() {
        let mut scd = vec![];
        scd.write_u16::<LE>(14).unwrap(); // Multi event 1.
        scd.write_u16::<LE>(0x10).unwrap(); // Dummy event ID.
        let timestamp = 0x123456789abcdef;
        scd.write_u64::<LE>(timestamp).unwrap();
        let data = &[0x12, 0x34];
        scd.extend(data);

        scd.write_u16::<LE>(12).unwrap(); // Multi event 2.
        scd.write_u16::<LE>(0x11).unwrap(); // Dummy event ID.
        let timestamp = 0x1;
        scd.write_u64::<LE>(timestamp).unwrap();

        let mut raw_packet = serialize_header(scd.len() as u16, 1);
        raw_packet.extend(scd);

        let event_packet = EventPacket::parse(&raw_packet).unwrap();

        assert_eq!(event_packet.request_id(), 1);
        assert_eq!(event_packet.scd.len(), 2);
        assert_eq!(event_packet.scd[0].event_id, 0x10);
        assert_eq!(event_packet.scd[0].timestamp, 0x123456789abcdef);
        assert_eq!(event_packet.scd[0].data, data);

        assert_eq!(event_packet.scd[1].event_id, 0x11);
        assert_eq!(event_packet.scd[1].timestamp, 0x1);
        assert_eq!(event_packet.scd[1].data, &[]);
    }
}
