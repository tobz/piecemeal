//! Traits and helpers for writing encoded Protocol Buffers messages and their fields.

use byteorder::{LittleEndian as LE, WriteBytesExt};

use crate::ProtoResult;

/// A Protocol Buffers-specific writer.
///
/// Provides methods for writing various types of fields and values, as well as for writing entire messages.
pub trait Writer {
    /// Write a u8
    fn pb_write_u8(&mut self, x: u8) -> ProtoResult<()>;

    /// Write a u32
    fn pb_write_u32(&mut self, x: u32) -> ProtoResult<()>;

    /// Write a i32
    fn pb_write_i32(&mut self, x: i32) -> ProtoResult<()>;

    /// Write a f32
    fn pb_write_f32(&mut self, x: f32) -> ProtoResult<()>;

    /// Write a u64
    fn pb_write_u64(&mut self, x: u64) -> ProtoResult<()>;

    /// Write a i64
    fn pb_write_i64(&mut self, x: i64) -> ProtoResult<()>;

    /// Write a f64
    fn pb_write_f64(&mut self, x: f64) -> ProtoResult<()>;

    /// Write all bytes in buf
    fn pb_write_all(&mut self, buf: &[u8]) -> ProtoResult<()>;

    /// Writes a `varint` (compacted `u64`)
    fn write_varint(&mut self, mut v: u64) -> ProtoResult<()> {
        while v > 0x7F {
            self.pb_write_u8(((v as u8) & 0x7F) | 0x80)?;
            v >>= 7;
        }
        self.pb_write_u8(v as u8)
    }

    /// Writes a tag, which represents both the field number and the wire type
    fn write_tag(&mut self, tag: u32) -> ProtoResult<()> {
        self.write_varint(tag as u64)
    }

    /// Writes a `int32` which is internally coded as a `varint`
    fn write_int32(&mut self, v: i32) -> ProtoResult<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `int64` which is internally coded as a `varint`
    fn write_int64(&mut self, v: i64) -> ProtoResult<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `uint32` which is internally coded as a `varint`
    fn write_uint32(&mut self, v: u32) -> ProtoResult<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `uint64` which is internally coded as a `varint`
    fn write_uint64(&mut self, v: u64) -> ProtoResult<()> {
        self.write_varint(v)
    }

    /// Writes a `sint32` which is internally coded as a `varint`
    fn write_sint32(&mut self, v: i32) -> ProtoResult<()> {
        self.write_varint(((v << 1) ^ (v >> 31)) as u64)
    }

    /// Writes a `sint64` which is internally coded as a `varint`
    fn write_sint64(&mut self, v: i64) -> ProtoResult<()> {
        self.write_varint(((v << 1) ^ (v >> 63)) as u64)
    }

    /// Writes a `fixed64` which is little endian coded `u64`
    fn write_fixed64(&mut self, v: u64) -> ProtoResult<()> {
        self.pb_write_u64(v)
    }

    /// Writes a `fixed32` which is little endian coded `u32`
    fn write_fixed32(&mut self, v: u32) -> ProtoResult<()> {
        self.pb_write_u32(v)
    }

    /// Writes a `sfixed64` which is little endian coded `i64`
    fn write_sfixed64(&mut self, v: i64) -> ProtoResult<()> {
        self.pb_write_i64(v)
    }

    /// Writes a `sfixed32` which is little endian coded `i32`
    fn write_sfixed32(&mut self, v: i32) -> ProtoResult<()> {
        self.pb_write_i32(v)
    }

    /// Writes a `float`
    fn write_float(&mut self, v: f32) -> ProtoResult<()> {
        self.pb_write_f32(v)
    }

    /// Writes a `double`
    fn write_double(&mut self, v: f64) -> ProtoResult<()> {
        self.pb_write_f64(v)
    }

    /// Writes a `bool` 1 = true, 0 = false
    fn write_bool(&mut self, v: bool) -> ProtoResult<()> {
        self.pb_write_u8(u8::from(v))
    }

    /// Writes an `enum` converting it to a `i32` first
    fn write_enum(&mut self, v: i32) -> ProtoResult<()> {
        self.write_int32(v)
    }

    /// Writes `bytes`: length first then the chunk of data
    fn write_bytes(&mut self, bytes: &[u8]) -> ProtoResult<()> {
        self.write_varint(bytes.len() as u64)?;
        self.pb_write_all(bytes)
    }

    /// Writes `string`: length first then the chunk of data
    fn write_string(&mut self, s: &str) -> ProtoResult<()> {
        self.write_bytes(s.as_bytes())
    }

    /// Writes another item prefixed with tag
    fn write_with_tag<F>(&mut self, tag: u32, mut write: F) -> ProtoResult<()>
    where
        F: FnMut(&mut Self) -> ProtoResult<()>,
        Self: Sized,
    {
        self.write_tag(tag)?;
        write(self)
    }
}

impl<W> Writer for W
where
    W: std::io::Write,
{
    fn pb_write_u8(&mut self, x: u8) -> ProtoResult<()> {
        WriteBytesExt::write_u8(self, x).map_err(|e| e.into())
    }

    fn pb_write_u32(&mut self, x: u32) -> ProtoResult<()> {
        self.write_u32::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_i32(&mut self, x: i32) -> ProtoResult<()> {
        self.write_i32::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_f32(&mut self, x: f32) -> ProtoResult<()> {
        self.write_f32::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_u64(&mut self, x: u64) -> ProtoResult<()> {
        self.write_u64::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_i64(&mut self, x: i64) -> ProtoResult<()> {
        self.write_i64::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_f64(&mut self, x: f64) -> ProtoResult<()> {
        self.write_f64::<LE>(x).map_err(|e| e.into())
    }

    fn pb_write_all(&mut self, buf: &[u8]) -> ProtoResult<()> {
        self.write_all(buf).map_err(|e| e.into())
    }
}
