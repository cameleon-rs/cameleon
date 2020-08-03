use std::io::{Cursor, Seek, SeekFrom};

use crate::usb3::{Error, Result};

pub(super) fn read_bytes<'a>(cursor: &mut Cursor<&'a [u8]>, len: u16) -> Result<&'a [u8]> {
    let current_pos = cursor.position() as usize;

    let buf = cursor.get_ref();
    let end_pos = len as usize + current_pos;

    if buf.len() < end_pos {
        use std::io;

        let err = io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "buffer is smaller than specified scd length",
        );
        return Err(Error::BufferIoError(err));
    };

    let data = &buf[current_pos..end_pos];

    // Advance cursor by read length.
    cursor.seek(SeekFrom::Current(len.into()))?;

    Ok(data)
}
