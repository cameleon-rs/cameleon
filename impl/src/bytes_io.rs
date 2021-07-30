use std::{io, net::Ipv4Addr};

pub trait ReadBytes {
    fn read_bytes_be<T>(&mut self) -> io::Result<T>
    where
        T: BytesConvertible;

    fn read_bytes_le<T>(&mut self) -> io::Result<T>
    where
        T: BytesConvertible;
}

pub trait WriteBytes {
    fn write_bytes_be<T>(&mut self, value: T) -> io::Result<()>
    where
        T: BytesConvertible;

    fn write_bytes_le<T>(&mut self, value: T) -> io::Result<()>
    where
        T: BytesConvertible;
}

impl<R> ReadBytes for R
where
    R: io::Read,
{
    fn read_bytes_be<T>(&mut self) -> io::Result<T>
    where
        T: BytesConvertible,
    {
        T::read_bytes_be(self)
    }

    fn read_bytes_le<T>(&mut self) -> io::Result<T>
    where
        T: BytesConvertible,
    {
        T::read_bytes_le(self)
    }
}

impl<W> WriteBytes for W
where
    W: io::Write,
{
    fn write_bytes_be<T>(&mut self, value: T) -> io::Result<()>
    where
        T: BytesConvertible,
    {
        value.write_bytes_be(self)
    }

    fn write_bytes_le<T>(&mut self, value: T) -> io::Result<()>
    where
        T: BytesConvertible,
    {
        value.write_bytes_le(self)
    }
}

pub trait BytesConvertible {
    fn read_bytes_be<R>(buf: &mut R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read;

    fn read_bytes_le<R>(buf: &mut R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read;

    fn write_bytes_be<W>(self, buf: &mut W) -> io::Result<()>
    where
        Self: Sized,
        W: io::Write;

    fn write_bytes_le<W>(self, buf: &mut W) -> io::Result<()>
    where
        Self: Sized,
        W: io::Write;
}

macro_rules! impl_bytes_convertible {
    ($($ty:ty,)*) => {
        $(
            impl BytesConvertible for $ty {
                fn read_bytes_be<R>(buf: &mut R) -> io::Result<Self>
                where
                    R: io::Read,
                {
                    let mut tmp = [0; std::mem::size_of::<$ty>()];
                    buf.read_exact(&mut tmp)?;
                    Ok(<$ty>::from_be_bytes(tmp))
                }

                fn read_bytes_le<R>(buf: &mut R) -> io::Result<Self>
                where
                    R: io::Read,
                {
                    let mut tmp = [0; std::mem::size_of::<$ty>()];
                    buf.read_exact(&mut tmp)?;
                    Ok(<$ty>::from_le_bytes(tmp))
                }

                fn write_bytes_be<W>(self, buf: &mut W) -> io::Result<()>
                where
                    W: io::Write,
                {
                    let tmp = self.to_be_bytes();
                    buf.write_all(&tmp)
                }

                fn write_bytes_le<W>(self, buf: &mut W) -> io::Result<()>
                where
                    W: io::Write,
                {
                    let tmp = self.to_le_bytes();
                    buf.write_all(&tmp)
                }
            }
        )*
    };
}

impl_bytes_convertible! {
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
}

impl BytesConvertible for Ipv4Addr {
    fn read_bytes_be<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut tmp = [0; 4];
        buf.read_exact(&mut tmp)?;
        Ok(tmp.into())
    }

    fn read_bytes_le<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut tmp = [0; 4];
        buf.read_exact(&mut tmp)?;
        Ok(tmp.into())
    }

    fn write_bytes_be<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let raw = self.octets();
        buf.write_all(&raw)?;
        Ok(())
    }

    fn write_bytes_le<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let raw = self.octets();
        buf.write_all(&raw)?;
        Ok(())
    }
}

impl<const N: usize> BytesConvertible for [u8; N] {
    fn read_bytes_be<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut res = [0; N];
        buf.read_exact(&mut res)?;
        Ok(res)
    }

    fn read_bytes_le<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        Self::read_bytes_be(buf)
    }

    fn write_bytes_be<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        buf.write_all(&self)?;
        Ok(())
    }

    fn write_bytes_le<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.write_bytes_be(buf)
    }
}

pub struct StaticString<const N: usize>(String);

impl<const N: usize> StaticString<N> {
    pub fn from_string(s: String) -> io::Result<Self> {
        if !s.is_ascii() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the data must be an ascii string",
            ));
        }

        if s.len() > N {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("string length exceeds {}", N),
            ));
        }

        Ok(Self(s))
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl<const N: usize> BytesConvertible for StaticString<N> {
    fn read_bytes_be<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut tmp = [0; N];
        buf.read_exact(&mut tmp)?;
        let str_end = tmp.iter().position(|c| *c == 0).unwrap_or(N);

        let s = String::from_utf8_lossy(&tmp[0..str_end]);
        Ok(Self(s.to_string()))
    }

    fn read_bytes_le<R>(buf: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        Self::read_bytes_be(buf)
    }

    fn write_bytes_be<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let mut bytes = self.0.into_bytes();
        bytes.resize(N, 0);
        buf.write_all(&bytes)
    }

    fn write_bytes_le<W>(self, buf: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.write_bytes_be(buf)
    }
}
