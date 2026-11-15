//! The smallest runtime the generated code needs (see the backend's module
//! docs), modelled on the one rose-proto provides.

pub type BString = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    UnexpectedEnd,
    UnknownCase { what: &'static str, value: i64 },
}

impl WireError {
    pub fn unknown_case(what: &'static str, value: i64) -> Self {
        WireError::UnknownCase { what, value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub u16);

#[allow(non_upper_case_globals)]
impl PacketType {
    pub const PAKCC_CLAN_COMMAND: Self = Self(0x7e0);
    pub const PAKWC_EVENT: Self = Self(0x7e1);
    pub const PAKWC_SHAPE: Self = Self(0x7e2);
    pub const PAKWC_WISH_LIST: Self = Self(0x7e3);
}

pub trait Packet: Wire {
    const TYPE: PacketType;
    const NAME: &'static str;
}

#[derive(Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn put_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    pub fn put_bits(&mut self, value: u64, bytes: usize) {
        self.0.extend_from_slice(&value.to_le_bytes()[..bytes]);
    }
}

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], WireError> {
        if n > self.buf.len() - self.pos {
            return Err(WireError::UnexpectedEnd);
        }
        self.pos += n;
        Ok(&self.buf[self.pos - n..self.pos])
    }

    pub fn get_bits(&mut self, bytes: usize) -> Result<u64, WireError> {
        let mut raw = [0u8; 8];
        raw[..bytes].copy_from_slice(self.take(bytes)?);
        Ok(u64::from_le_bytes(raw))
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn read_to_end<T: Wire>(&mut self) -> Vec<T> {
        let mut out = Vec::new();
        loop {
            let start = self.pos;
            match T::read(self) {
                Ok(value) if self.pos > start => out.push(value),
                _ => {
                    self.pos = start;
                    return out;
                }
            }
        }
    }

    pub fn read_counted<T: Wire>(&mut self, count: usize) -> Result<Vec<T>, WireError> {
        (0..count).map(|_| T::read(self)).collect()
    }

    pub fn read_array<T: Wire, const N: usize>(&mut self) -> Result<[T; N], WireError> {
        let items = self.read_counted::<T>(N)?;
        Ok(items.try_into().unwrap_or_else(|_| unreachable!()))
    }
}

pub trait Wire: Sized {
    fn write(&self, w: &mut Writer);
    fn read(r: &mut Reader<'_>) -> Result<Self, WireError>;
}

macro_rules! wire_number {
    ($($t:ty),*) => {$(
        impl Wire for $t {
            fn write(&self, w: &mut Writer) {
                w.put_bytes(&self.to_le_bytes());
            }
            fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {
                Ok(<$t>::from_le_bytes(r.take(std::mem::size_of::<$t>())?.try_into().unwrap()))
            }
        }
    )*};
}
wire_number!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl Wire for BString {
    fn write(&self, w: &mut Writer) {
        w.put_bytes(self);
        w.put_bytes(&[0]);
    }

    fn read(r: &mut Reader<'_>) -> Result<Self, WireError> {
        let rest = &r.buf[r.pos..];
        let end = rest.iter().position(|&b| b == 0).ok_or(WireError::UnexpectedEnd)?;
        r.pos += end + 1;
        Ok(rest[..end].to_vec())
    }
}
