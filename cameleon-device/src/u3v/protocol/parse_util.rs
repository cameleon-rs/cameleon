use std::io::{self, Seek};

use crate::u3v::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};

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

pub(crate) trait ReadBytes {
    fn read_bytes<T>(&mut self) -> Result<T>
    where
        T: BytesConvertible;
}

pub(crate) trait WriteBytes {
    fn write_bytes<T>(&mut self, value: T) -> Result<()>
    where
        T: BytesConvertible;
}

impl<T> ReadBytes for T
where
    T: std::io::Read,
{
    fn read_bytes<U>(&mut self) -> Result<U>
    where
        U: BytesConvertible,
    {
        U::read_bytes(self)
    }
}

impl<T> WriteBytes for T
where
    T: std::io::Write,
{
    fn write_bytes<U>(&mut self, value: U) -> Result<()>
    where
        U: BytesConvertible,
    {
        value.write_bytes(self)
    }
}

pub(crate) trait BytesConvertible {
    fn read_bytes<T>(buf: &mut T) -> Result<Self>
    where
        Self: Sized,
        T: std::io::Read;

    fn write_bytes<T>(self, buf: &mut T) -> Result<()>
    where
        Self: Sized,
        T: std::io::Write;
}

macro_rules! impl_parse_bytes {
    (u8, read_u8, write_u8) => {
        impl BytesConvertible for u8 {
            fn read_bytes<T>(buf: &mut T) -> Result<Self>
            where
                T: std::io::Read,
            {
                Ok(buf.read_u8()?)
            }

            fn write_bytes<T>(self, buf: &mut T) -> Result<()>
            where
                T: std::io::Write,
            {
                Ok(buf.write_u8(self)?)
            }
        }
    };

    (i8, read_i8, write_i8) => {
        impl BytesConvertible for i8 {
            fn read_bytes<T>(buf: &mut T) -> Result<Self>
            where
                T: std::io::Read,
            {
                Ok(buf.read_i8()?)
            }

            fn write_bytes<T>(self, buf: &mut T) -> Result<()>
            where
                T: std::io::Write,
            {
                Ok(buf.write_i8(self)?)
            }
        }
    };

    ($ty:ty, $read_method:ident, $write_method:ident) => {
        impl BytesConvertible for $ty {
            fn read_bytes<T>(buf: &mut T) -> Result<Self>
            where
                T: std::io::Read,
            {
                Ok(buf.$read_method::<LE>()?)
            }

            fn write_bytes<T>(self, buf: &mut T) -> Result<()>
            where
                T: std::io::Write,
            {
                Ok(buf.$write_method::<LE>(self)?)
            }
        }
    };
}

impl_parse_bytes!(u8, read_u8, write_u8);
impl_parse_bytes!(u16, read_u16, write_u16);
impl_parse_bytes!(u32, read_u32, write_u32);
impl_parse_bytes!(u64, read_u64, write_u64);
impl_parse_bytes!(i8, read_i8, write_i8);
impl_parse_bytes!(i16, read_i16, write_i16);
impl_parse_bytes!(i32, read_i32, write_i32);
impl_parse_bytes!(i64, read_i64, write_i64);
