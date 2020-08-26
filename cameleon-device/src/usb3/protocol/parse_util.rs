use std::io::{self, Seek};

pub(crate) fn read_bytes<'a>(cursor: &mut io::Cursor<&'a [u8]>, len: u16) -> io::Result<&'a [u8]> {
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

    // Advance cursor by read length.
    cursor.seek(io::SeekFrom::Current(len.into()))?;

    Ok(data)
}
