/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{self, Cursor};

use cameleon_impl::bytes_io::ReadBytes;

use crate::u3v::{Error, Result};

pub struct EventPacket<'a> {
    ccd: EventCcd,
    pub scd: Vec<EventScd<'a>>,
}

impl<'a> EventPacket<'a> {
    const PREFIX_MAGIC: u32 = 0x4556_3355;

    pub fn parse(buf: &'a (impl AsRef<[u8]> + ?Sized)) -> Result<Self> {
        let mut cursor = Cursor::new(buf.as_ref());

        Self::parse_prefix(&mut cursor)?;

        let ccd = EventCcd::parse(&mut cursor)?;

        let scd = EventScd::parse(&mut cursor, &ccd)?;

        Ok(Self { ccd, scd })
    }

    #[must_use]
    pub fn request_id(&self) -> u16 {
        self.ccd.request_id
    }

    fn parse_prefix(cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let magic: u32 = cursor.read_bytes_le()?;
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
        let flag = cursor.read_bytes_le()?;
        let command_id = cursor.read_bytes_le()?;
        if command_id != Self::EVENT_COMMAND_ID {
            return Err(Error::InvalidPacket("invalid event command id".into()));
        }
        let scd_len = cursor.read_bytes_le()?;
        let request_id = cursor.read_bytes_le()?;
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
        fn read_and_seek<'a>(cursor: &mut io::Cursor<&'a [u8]>, len: u16) -> io::Result<&'a [u8]> {
            use std::io::Seek;
            let current_pos = cursor.position() as usize;
            let buf = cursor.get_ref();
            let end_pos = len as usize + current_pos;

            if buf.len() < end_pos {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "data is smaller than specified length",
                ));
            };

            let data = &buf[current_pos..end_pos];
            cursor.seek(io::SeekFrom::Current(len.into()))?;
            Ok(data)
        }

        let mut events = vec![];
        let mut remained = ccd.scd_len;

        while remained > 0 {
            let event_size: u16 = cursor.read_bytes_le()?;
            let event_id = cursor.read_bytes_le()?;
            let timestamp = cursor.read_bytes_le()?;

            // MultiEvent isn't enabled.
            let data = if event_size == 0 {
                remained = remained.checked_sub(12).ok_or_else(|| {
                    Error::InvalidPacket("SCD length in CCD is inconsistent with SCD".into())
                })?;
                let data = read_and_seek(cursor, remained)?;
                remained = 0;
                data
            } else {
                let data_len = event_size.checked_sub(12).ok_or_else(|| {
                    Error::InvalidPacket("event size is smaller than scd header".into())
                })?;
                remained = remained.checked_sub(event_size).ok_or_else(|| {
                    Error::InvalidPacket("SCD length in CCD is inconsistent with SCD".into())
                })?;
                read_and_seek(cursor, data_len)?
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
    use super::*;

    use cameleon_impl::bytes_io::WriteBytes;

    fn serialize_header(scd_len: u16, request_id: u16) -> Vec<u8> {
        let mut ccd = vec![];
        ccd.write_bytes_le(0x4556_3355_u32).unwrap(); // Magic.
        ccd.write_bytes_le(1_u16 << 14).unwrap(); // Request ack for now.
        ccd.write_bytes_le(0x0c00_u16).unwrap();
        ccd.write_bytes_le(scd_len).unwrap();
        ccd.write_bytes_le(request_id).unwrap();
        ccd
    }

    #[test]
    fn test_single_event() {
        let mut scd = vec![];
        scd.write_bytes_le(0_u16).unwrap(); // Single event.
        scd.write_bytes_le(0x10_u16).unwrap(); // Dummy event ID.
        let timestamp = 0x0123_4567_89ab_cdef_u64;
        scd.write_bytes_le(timestamp).unwrap();
        let data = &[0x12, 0x34];
        scd.extend(data);

        let mut raw_packet = serialize_header(scd.len() as u16, 1);
        raw_packet.extend(scd);

        let event_packet = EventPacket::parse(&raw_packet).unwrap();

        assert_eq!(event_packet.request_id(), 1);
        assert_eq!(event_packet.scd.len(), 1);
        assert_eq!(event_packet.scd[0].event_id, 0x10);
        assert_eq!(event_packet.scd[0].timestamp, 0x0123_4567_89ab_cdef);
        assert_eq!(event_packet.scd[0].data, data);
    }

    #[test]
    fn test_multi_event() {
        let mut scd = vec![];
        scd.write_bytes_le(14_u16).unwrap(); // Multi event 1.
        scd.write_bytes_le(0x10_u16).unwrap(); // Dummy event ID.
        let timestamp1 = 0x0123_4567_89ab_cdef_u64;
        scd.write_bytes_le(timestamp1).unwrap();
        let data = &[0x12, 0x34];
        scd.extend(data);

        scd.write_bytes_le(12_u16).unwrap(); // Multi event 2.
        scd.write_bytes_le(0x11_u16).unwrap(); // Dummy event ID.
        let timestamp2 = 0x1_u64;
        scd.write_bytes_le(timestamp2).unwrap();

        let mut raw_packet = serialize_header(scd.len() as u16, 1);
        raw_packet.extend(scd);

        let event_packet = EventPacket::parse(&raw_packet).unwrap();

        assert_eq!(event_packet.request_id(), 1);
        assert_eq!(event_packet.scd.len(), 2);
        assert_eq!(event_packet.scd[0].event_id, 0x10);
        assert_eq!(event_packet.scd[0].timestamp, timestamp1);
        assert_eq!(event_packet.scd[0].data, data);

        assert_eq!(event_packet.scd[1].event_id, 0x11);
        assert_eq!(event_packet.scd[1].timestamp, timestamp2);
        assert_eq!(event_packet.scd[1].data, &[]);
    }
}
