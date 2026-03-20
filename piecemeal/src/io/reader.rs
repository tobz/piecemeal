
//! Types and helpers for reading (decoding) Protocol Buffers messages and their fields.

use std::fmt;

use byteorder::{ByteOrder, LittleEndian as LE};

use crate::types::WireType;

/// Errors that can occur while decoding Protocol Buffers data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The input buffer was exhausted before a complete value could be read.
    UnexpectedEof,
    /// A varint exceeded the maximum of 10 bytes.
    VarintOverflow,
    /// An unknown wire type was encountered in a tag.
    UnknownWireType(u32),
    /// A length-delimited field's length exceeds the remaining buffer.
    LengthOverflow {
        /// The declared length of the field.
        len: usize,
        /// The number of bytes remaining in the buffer.
        remaining: usize,
    },
    /// A string field contained invalid UTF-8.
    InvalidUtf8,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::VarintOverflow => write!(f, "varint exceeded 10 bytes"),
            Self::UnknownWireType(wt) => write!(f, "unknown wire type: {wt}"),
            Self::LengthOverflow { len, remaining } => {
                write!(f, "length {len} exceeds remaining {remaining} bytes")
            }
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in string field"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Raw field data for unknown or skipped fields.
#[derive(Debug, Clone, PartialEq)]
pub enum RawField<'a> {
    /// A varint-encoded value.
    Varint(u64),
    /// A 32-bit fixed-width value.
    Fixed32([u8; 4]),
    /// A 64-bit fixed-width value.
    Fixed64([u8; 8]),
    /// A length-delimited value (bytes, string, embedded message, or packed repeated).
    LengthDelimited(&'a [u8]),
}

/// Records the location(s) of a singular message field within a buffer.
///
/// Used during the initial decode pass to record where embedded message bytes live,
/// so they can be decoded lazily when accessed.
///
/// Optimized for the common case of a single occurrence (zero allocation).
/// The rare case of multiple occurrences (requiring merge) uses a `Vec`.
#[derive(Debug, Clone)]
pub enum FieldSlice {
    /// The field was not present in the wire data.
    None,
    /// The field appeared exactly once (common case, zero allocation).
    One {
        /// Byte offset into the parent buffer where the message payload starts.
        offset: u32,
        /// Length in bytes of the message payload.
        len: u32,
    },
    /// The field appeared multiple times (rare, requires merging).
    Many(Vec<(u32, u32)>),
}

impl FieldSlice {
    /// Records an occurrence of the field at the given offset and length.
    ///
    /// Transitions: `None → One → Many`.
    pub fn record(&mut self, offset: u32, len: u32) {
        match self {
            FieldSlice::None => *self = FieldSlice::One { offset, len },
            FieldSlice::One {
                offset: o,
                len: l,
            } => {
                *self = FieldSlice::Many(vec![(*o, *l), (offset, len)]);
            }
            FieldSlice::Many(v) => v.push((offset, len)),
        }
    }
}

/// A zero-copy reader over a byte slice for decoding Protocol Buffers wire format.
///
/// This is the dual of [`Writer`](crate::Writer) — where `Writer` pushes fields into a byte
/// stream, `Reader` pulls fields from a byte slice.
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Creates a new reader over the given byte slice.
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns `true` if there are no more bytes to read.
    pub fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    /// Returns the current byte position within the buffer.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of remaining unread bytes.
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    // --- Low-level reads ---

    /// Reads a raw varint as `u64`.
    pub fn read_varint(&mut self) -> Result<u64, DecodeError> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            if self.pos >= self.buf.len() {
                return Err(DecodeError::UnexpectedEof);
            }
            let byte = self.buf[self.pos];
            self.pos += 1;

            if shift >= 63 && byte > 1 {
                return Err(DecodeError::VarintOverflow);
            }

            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                return Ok(result);
            }

            shift += 7;
            if shift >= 70 {
                return Err(DecodeError::VarintOverflow);
            }
        }
    }

    /// Reads a tag, returning the field number and wire type.
    pub fn read_tag(&mut self) -> Result<(u32, WireType), DecodeError> {
        let v = self.read_varint()?;
        let field_number = (v >> 3) as u32;
        let wire_type_raw = (v & 0x07) as u32;
        let wire_type =
            WireType::from_u32(wire_type_raw).ok_or(DecodeError::UnknownWireType(wire_type_raw))?;
        Ok((field_number, wire_type))
    }

    // --- Typed scalar reads (dual of each Writer::write_* method) ---

    /// Reads an `int32` (varint-encoded, sign-extended to 64 bits on the wire).
    pub fn read_int32(&mut self) -> Result<i32, DecodeError> {
        self.read_varint().map(|v| v as i32)
    }

    /// Reads an `int64` (varint-encoded).
    pub fn read_int64(&mut self) -> Result<i64, DecodeError> {
        self.read_varint().map(|v| v as i64)
    }

    /// Reads a `uint32` (varint-encoded).
    pub fn read_uint32(&mut self) -> Result<u32, DecodeError> {
        self.read_varint().map(|v| v as u32)
    }

    /// Reads a `uint64` (varint-encoded).
    pub fn read_uint64(&mut self) -> Result<u64, DecodeError> {
        self.read_varint()
    }

    /// Reads a `sint32` (zigzag-encoded varint).
    pub fn read_sint32(&mut self) -> Result<i32, DecodeError> {
        let v = self.read_varint()? as u32;
        Ok(((v >> 1) as i32) ^ (-((v & 1) as i32)))
    }

    /// Reads a `sint64` (zigzag-encoded varint).
    pub fn read_sint64(&mut self) -> Result<i64, DecodeError> {
        let v = self.read_varint()?;
        Ok(((v >> 1) as i64) ^ (-((v & 1) as i64)))
    }

    /// Reads a `bool` (varint-encoded, 0 = false, nonzero = true).
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        self.read_varint().map(|v| v != 0)
    }

    /// Reads an `enum` value as `i32` (varint-encoded).
    pub fn read_enum(&mut self) -> Result<i32, DecodeError> {
        self.read_int32()
    }

    /// Reads a `fixed32` (4 bytes, little-endian).
    pub fn read_fixed32(&mut self) -> Result<u32, DecodeError> {
        let bytes = self.read_exact(4)?;
        Ok(LE::read_u32(bytes))
    }

    /// Reads a `fixed64` (8 bytes, little-endian).
    pub fn read_fixed64(&mut self) -> Result<u64, DecodeError> {
        let bytes = self.read_exact(8)?;
        Ok(LE::read_u64(bytes))
    }

    /// Reads an `sfixed32` (4 bytes, little-endian, signed).
    pub fn read_sfixed32(&mut self) -> Result<i32, DecodeError> {
        let bytes = self.read_exact(4)?;
        Ok(LE::read_i32(bytes))
    }

    /// Reads an `sfixed64` (8 bytes, little-endian, signed).
    pub fn read_sfixed64(&mut self) -> Result<i64, DecodeError> {
        let bytes = self.read_exact(8)?;
        Ok(LE::read_i64(bytes))
    }

    /// Reads a `float` (4 bytes, little-endian).
    pub fn read_float(&mut self) -> Result<f32, DecodeError> {
        let bytes = self.read_exact(4)?;
        Ok(LE::read_f32(bytes))
    }

    /// Reads a `double` (8 bytes, little-endian).
    pub fn read_double(&mut self) -> Result<f64, DecodeError> {
        let bytes = self.read_exact(8)?;
        Ok(LE::read_f64(bytes))
    }

    // --- Length-delimited reads ---

    /// Reads a length-delimited field, returning the sub-slice of bytes.
    ///
    /// Used for embedded messages, packed repeated fields, bytes, and strings.
    pub fn read_length_delimited(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.buf.len() {
            return Err(DecodeError::LengthOverflow {
                len,
                remaining: self.remaining(),
            });
        }
        let data = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(data)
    }

    /// Reads a `string` (length-delimited, UTF-8 validated).
    pub fn read_string(&mut self) -> Result<&'a str, DecodeError> {
        let bytes = self.read_length_delimited()?;
        std::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
    }

    /// Reads a `bytes` field (length-delimited).
    pub fn read_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        self.read_length_delimited()
    }

    // --- Field skipping ---

    /// Skips a field of the given wire type, advancing past it without decoding.
    pub fn skip_field(&mut self, wire_type: WireType) -> Result<(), DecodeError> {
        match wire_type {
            WireType::Varint => {
                self.read_varint()?;
            }
            WireType::Fixed64 => {
                self.skip_bytes(8)?;
            }
            WireType::LengthDelimited => {
                let len = self.read_varint()? as usize;
                self.skip_bytes(len)?;
            }
            WireType::Fixed32 => {
                self.skip_bytes(4)?;
            }
        }
        Ok(())
    }

    /// Advances past `n` bytes without decoding them.
    pub fn skip_bytes(&mut self, n: usize) -> Result<(), DecodeError> {
        if self.pos + n > self.buf.len() {
            return Err(DecodeError::LengthOverflow {
                len: n,
                remaining: self.remaining(),
            });
        }
        self.pos += n;
        Ok(())
    }

    /// Reads a raw field value for the given wire type (for unknown fields).
    pub fn read_raw_field(&mut self, wire_type: WireType) -> Result<RawField<'a>, DecodeError> {
        match wire_type {
            WireType::Varint => self.read_varint().map(RawField::Varint),
            WireType::Fixed32 => {
                let bytes = self.read_exact(4)?;
                let mut arr = [0u8; 4];
                arr.copy_from_slice(bytes);
                Ok(RawField::Fixed32(arr))
            }
            WireType::Fixed64 => {
                let bytes = self.read_exact(8)?;
                let mut arr = [0u8; 8];
                arr.copy_from_slice(bytes);
                Ok(RawField::Fixed64(arr))
            }
            WireType::LengthDelimited => {
                self.read_length_delimited().map(RawField::LengthDelimited)
            }
        }
    }

    // --- Internal helpers ---

    /// Reads exactly `n` bytes from the buffer, returning a sub-slice.
    fn read_exact(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.pos + n > self.buf.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let data = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Writer;

    // --- Reader: varint ---

    #[test]
    fn read_varint_single_byte() {
        let mut buf = Vec::new();
        buf.write_varint(0).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert!(reader.is_empty());
    }

    #[test]
    fn read_varint_multi_byte() {
        let mut buf = Vec::new();
        buf.write_varint(300).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_varint().unwrap(), 300);
        assert!(reader.is_empty());
    }

    #[test]
    fn read_varint_max() {
        let mut buf = Vec::new();
        buf.write_varint(u64::MAX).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_varint().unwrap(), u64::MAX);
        assert!(reader.is_empty());
    }

    #[test]
    fn read_varint_unexpected_eof() {
        let buf = [0x80]; // continuation bit set but no more bytes
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_varint().unwrap_err(), DecodeError::UnexpectedEof);
    }

    #[test]
    fn read_varint_overflow() {
        // 11 bytes of 0x80 followed by 0x01 — too many bytes for a varint
        let buf = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let mut reader = Reader::new(&buf);
        assert_eq!(
            reader.read_varint().unwrap_err(),
            DecodeError::VarintOverflow
        );
    }

    // --- Reader: tag ---

    #[test]
    fn read_tag_roundtrip() {
        use crate::helpers::tag;
        let mut buf = Vec::new();
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
        buf.write_tag(tag(15, WireType::Fixed32)).unwrap();
        buf.write_tag(tag(16, WireType::Fixed64)).unwrap();

        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_tag().unwrap(), (1, WireType::Varint));
        assert_eq!(reader.read_tag().unwrap(), (2, WireType::LengthDelimited));
        assert_eq!(reader.read_tag().unwrap(), (15, WireType::Fixed32));
        assert_eq!(reader.read_tag().unwrap(), (16, WireType::Fixed64));
        assert!(reader.is_empty());
    }

    // --- Reader: typed scalar roundtrips ---

    #[test]
    fn read_int32_roundtrip() {
        for &v in &[0i32, 1, -1, 127, -128, i32::MAX, i32::MIN] {
            let mut buf = Vec::new();
            buf.write_int32(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_int32().unwrap(), v, "roundtrip failed for {v}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_int64_roundtrip() {
        for &v in &[0i64, 1, -1, i64::MAX, i64::MIN] {
            let mut buf = Vec::new();
            buf.write_int64(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_int64().unwrap(), v, "roundtrip failed for {v}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_uint32_roundtrip() {
        for &v in &[0u32, 1, 127, 128, u32::MAX] {
            let mut buf = Vec::new();
            buf.write_uint32(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_uint32().unwrap(), v, "roundtrip failed for {v}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_uint64_roundtrip() {
        for &v in &[0u64, 1, 127, 128, u64::MAX] {
            let mut buf = Vec::new();
            buf.write_uint64(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_uint64().unwrap(), v, "roundtrip failed for {v}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_sint32_roundtrip() {
        for &v in &[0i32, 1, -1, 63, -64, i32::MAX, i32::MIN] {
            let mut buf = Vec::new();
            buf.write_sint32(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_sint32().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_sint64_roundtrip() {
        for &v in &[0i64, 1, -1, 63, -64, i64::MAX, i64::MIN] {
            let mut buf = Vec::new();
            buf.write_sint64(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_sint64().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_bool_roundtrip() {
        for &v in &[true, false] {
            let mut buf = Vec::new();
            buf.write_bool(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_bool().unwrap(), v);
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_fixed32_roundtrip() {
        for &v in &[0u32, 1, u32::MAX] {
            let mut buf = Vec::new();
            buf.write_fixed32(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_fixed32().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_fixed64_roundtrip() {
        for &v in &[0u64, 1, u64::MAX] {
            let mut buf = Vec::new();
            buf.write_fixed64(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_fixed64().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_sfixed32_roundtrip() {
        for &v in &[0i32, 1, -1, i32::MAX, i32::MIN] {
            let mut buf = Vec::new();
            buf.write_sfixed32(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_sfixed32().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_sfixed64_roundtrip() {
        for &v in &[0i64, 1, -1, i64::MAX, i64::MIN] {
            let mut buf = Vec::new();
            buf.write_sfixed64(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_sfixed64().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_float_roundtrip() {
        for &v in &[0.0f32, 1.0, -1.0, f32::MAX, f32::MIN, f32::INFINITY] {
            let mut buf = Vec::new();
            buf.write_float(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_float().unwrap(), v, "roundtrip failed for {v}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_float_nan() {
        let mut buf = Vec::new();
        buf.write_float(f32::NAN).unwrap();
        let mut reader = Reader::new(&buf);
        assert!(reader.read_float().unwrap().is_nan());
    }

    #[test]
    fn read_double_roundtrip() {
        for &v in &[0.0f64, 1.0, -1.0, f64::MAX, f64::MIN, f64::INFINITY] {
            let mut buf = Vec::new();
            buf.write_double(v).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(
                reader.read_double().unwrap(),
                v,
                "roundtrip failed for {v}"
            );
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_double_nan() {
        let mut buf = Vec::new();
        buf.write_double(f64::NAN).unwrap();
        let mut reader = Reader::new(&buf);
        assert!(reader.read_double().unwrap().is_nan());
    }

    // --- Reader: length-delimited ---

    #[test]
    fn read_string_roundtrip() {
        for &s in &["", "hello", "hello world", "a".repeat(128).as_str()] {
            let mut buf = Vec::new();
            buf.write_string(s).unwrap();
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_string().unwrap(), s, "roundtrip failed for {s}");
            assert!(reader.is_empty());
        }
    }

    #[test]
    fn read_string_invalid_utf8() {
        let mut buf = Vec::new();
        buf.write_bytes(&[0xFF, 0xFE]).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_string().unwrap_err(), DecodeError::InvalidUtf8);
    }

    #[test]
    fn read_bytes_roundtrip() {
        let data: &[u8] = &[1, 2, 3, 4, 5];
        let mut buf = Vec::new();
        buf.write_bytes(data).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_bytes().unwrap(), data);
        assert!(reader.is_empty());
    }

    #[test]
    fn read_length_delimited_overflow() {
        // Write a length of 100 but only provide 5 bytes of data
        let mut buf = Vec::new();
        buf.write_varint(100).unwrap();
        buf.extend_from_slice(&[0; 5]);
        let mut reader = Reader::new(&buf);
        assert!(matches!(
            reader.read_length_delimited().unwrap_err(),
            DecodeError::LengthOverflow { .. }
        ));
    }

    // --- Reader: skip ---

    #[test]
    fn skip_field_varint() {
        let mut buf = Vec::new();
        buf.write_varint(12345).unwrap();
        let mut reader = Reader::new(&buf);
        reader.skip_field(WireType::Varint).unwrap();
        assert!(reader.is_empty());
    }

    #[test]
    fn skip_field_fixed32() {
        let mut buf = Vec::new();
        buf.write_fixed32(42).unwrap();
        let mut reader = Reader::new(&buf);
        reader.skip_field(WireType::Fixed32).unwrap();
        assert!(reader.is_empty());
    }

    #[test]
    fn skip_field_fixed64() {
        let mut buf = Vec::new();
        buf.write_fixed64(42).unwrap();
        let mut reader = Reader::new(&buf);
        reader.skip_field(WireType::Fixed64).unwrap();
        assert!(reader.is_empty());
    }

    #[test]
    fn skip_field_length_delimited() {
        let mut buf = Vec::new();
        buf.write_string("hello world").unwrap();
        let mut reader = Reader::new(&buf);
        reader.skip_field(WireType::LengthDelimited).unwrap();
        assert!(reader.is_empty());
    }

    // --- Reader: multiple fields in sequence ---

    #[test]
    fn read_multiple_fields() {
        use crate::helpers::tag;

        let mut buf = Vec::new();
        // Field 1: int32 = 42
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(42).unwrap();
        // Field 2: string = "hello"
        buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
        buf.write_string("hello").unwrap();
        // Field 3: fixed64 = 999
        buf.write_tag(tag(3, WireType::Fixed64)).unwrap();
        buf.write_fixed64(999).unwrap();

        let mut reader = Reader::new(&buf);

        let (fn1, wt1) = reader.read_tag().unwrap();
        assert_eq!((fn1, wt1), (1, WireType::Varint));
        assert_eq!(reader.read_int32().unwrap(), 42);

        let (fn2, wt2) = reader.read_tag().unwrap();
        assert_eq!((fn2, wt2), (2, WireType::LengthDelimited));
        assert_eq!(reader.read_string().unwrap(), "hello");

        let (fn3, wt3) = reader.read_tag().unwrap();
        assert_eq!((fn3, wt3), (3, WireType::Fixed64));
        assert_eq!(reader.read_fixed64().unwrap(), 999);

        assert!(reader.is_empty());
    }

    // --- Reader: raw field ---

    #[test]
    fn read_raw_field_varint() {
        let mut buf = Vec::new();
        buf.write_varint(42).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(
            reader.read_raw_field(WireType::Varint).unwrap(),
            RawField::Varint(42)
        );
    }

    #[test]
    fn read_raw_field_fixed32() {
        let mut buf = Vec::new();
        buf.write_fixed32(42).unwrap();
        let mut reader = Reader::new(&buf);
        let raw = reader.read_raw_field(WireType::Fixed32).unwrap();
        assert!(matches!(raw, RawField::Fixed32(_)));
    }

    #[test]
    fn read_raw_field_length_delimited() {
        let mut buf = Vec::new();
        buf.write_bytes(&[1, 2, 3]).unwrap();
        let mut reader = Reader::new(&buf);
        assert_eq!(
            reader.read_raw_field(WireType::LengthDelimited).unwrap(),
            RawField::LengthDelimited(&[1, 2, 3])
        );
    }

    // --- FieldSlice ---

    #[test]
    fn field_slice_transitions() {
        let mut fs = FieldSlice::None;
        assert!(matches!(fs, FieldSlice::None));

        fs.record(10, 20);
        assert!(matches!(fs, FieldSlice::One { offset: 10, len: 20 }));

        fs.record(50, 30);
        match &fs {
            FieldSlice::Many(v) => assert_eq!(v, &[(10, 20), (50, 30)]),
            _ => panic!("expected Many"),
        }

        fs.record(100, 5);
        match &fs {
            FieldSlice::Many(v) => assert_eq!(v, &[(10, 20), (50, 30), (100, 5)]),
            _ => panic!("expected Many"),
        }
    }
}
