//! Zero-allocation iterators for repeated and map protobuf fields.
//!
//! These iterators re-scan a message's byte buffer to lazily yield field values,
//! avoiding upfront allocation for repeated and map fields.

use crate::types::WireType;

use super::reader::{DecodeError, Reader};

/// An iterator over repeated non-packable fields (strings, bytes, messages).
///
/// Scans the message buffer looking for fields matching a specific field number,
/// decoding each one via a function pointer.
pub struct FieldIter<'a, T> {
    reader: Reader<'a>,
    field_number: u32,
    decode_fn: fn(&mut Reader<'a>) -> Result<T, DecodeError>,
}

impl<'a, T> FieldIter<'a, T> {
    /// Creates a new `FieldIter` that scans `buf` for the given `field_number`,
    /// expecting `WireType::LengthDelimited` encoding, and decoding values with `decode_fn`.
    pub fn new(
        buf: &'a [u8],
        field_number: u32,
        decode_fn: fn(&mut Reader<'a>) -> Result<T, DecodeError>,
    ) -> Self {
        Self {
            reader: Reader::new(buf),
            field_number,
            decode_fn,
        }
    }
}

impl<'a, T> Iterator for FieldIter<'a, T> {
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.reader.is_empty() {
            let (field_number, wire_type) = match self.reader.read_tag() {
                Ok(tag) => tag,
                Err(e) => return Some(Err(e)),
            };

            if field_number == self.field_number && wire_type == WireType::LengthDelimited {
                return Some((self.decode_fn)(&mut self.reader));
            }

            // Not our field — skip it.
            if let Err(e) = self.reader.skip_field(wire_type) {
                return Some(Err(e));
            }
        }
        None
    }
}

/// An iterator over repeated packable fields (numeric types, bool, enum).
///
/// Handles both packed encoding (wire type 2 = length-delimited blob of concatenated values)
/// and unpacked encoding (individual tagged values) transparently.
pub struct PackedFieldIter<'a, T> {
    reader: Reader<'a>,
    field_number: u32,
    /// The native wire type for individual (unpacked) values.
    element_wire_type: WireType,
    decode_fn: fn(&mut Reader<'a>) -> Result<T, DecodeError>,
    /// When draining a packed block, holds the sub-reader over the packed bytes.
    packed_reader: Option<Reader<'a>>,
}

impl<'a, T> PackedFieldIter<'a, T> {
    /// Creates a new `PackedFieldIter`.
    ///
    /// - `element_wire_type`: the wire type for individual (unpacked) elements
    ///   (e.g., `Varint` for int32, `Fixed32` for float).
    pub fn new(
        buf: &'a [u8],
        field_number: u32,
        element_wire_type: WireType,
        decode_fn: fn(&mut Reader<'a>) -> Result<T, DecodeError>,
    ) -> Self {
        Self {
            reader: Reader::new(buf),
            field_number,
            element_wire_type,
            decode_fn,
            packed_reader: None,
        }
    }
}

impl<'a, T> Iterator for PackedFieldIter<'a, T> {
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we're mid-packed-block, drain it first.
        if let Some(ref mut packed) = self.packed_reader {
            if !packed.is_empty() {
                return Some((self.decode_fn)(packed));
            }
            self.packed_reader = None;
        }

        while !self.reader.is_empty() {
            let (field_number, wire_type) = match self.reader.read_tag() {
                Ok(tag) => tag,
                Err(e) => return Some(Err(e)),
            };

            if field_number == self.field_number {
                if wire_type == self.element_wire_type {
                    // Unpacked individual value.
                    return Some((self.decode_fn)(&mut self.reader));
                } else if wire_type == WireType::LengthDelimited {
                    // Packed encoding: read the blob, create sub-reader.
                    let packed_bytes = match self.reader.read_length_delimited() {
                        Ok(b) => b,
                        Err(e) => return Some(Err(e)),
                    };
                    if packed_bytes.is_empty() {
                        continue; // Empty packed field, skip.
                    }
                    let mut packed = Reader::new(packed_bytes);
                    let result = (self.decode_fn)(&mut packed);
                    if !packed.is_empty() {
                        self.packed_reader = Some(packed);
                    }
                    return Some(result);
                }
                // Wire type mismatch for our field — skip it.
            }

            // Not our field — skip it.
            if let Err(e) = self.reader.skip_field(wire_type) {
                return Some(Err(e));
            }
        }
        None
    }
}

/// An iterator over map field entries, yielding `(K, V)` pairs.
///
/// Maps are wire-encoded as repeated length-delimited fields, where each entry
/// is a small sub-message with key = field 1 and value = field 2.
pub struct MapIter<'a, K, V> {
    inner: FieldIter<'a, (K, V)>,
}

impl<'a, K, V> MapIter<'a, K, V> {
    /// Creates a new `MapIter`.
    ///
    /// The `decode_fn` should read a length-delimited map entry and return `(key, value)`.
    pub fn new(
        buf: &'a [u8],
        field_number: u32,
        decode_fn: fn(&mut Reader<'a>) -> Result<(K, V), DecodeError>,
    ) -> Self {
        Self {
            inner: FieldIter::new(buf, field_number, decode_fn),
        }
    }
}

impl<'a, K, V> Iterator for MapIter<'a, K, V> {
    type Item = Result<(K, V), DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Writer, helpers::tag};

    // Helper: encode a message with repeated int32 field (field 1, unpacked)
    fn encode_repeated_int32_unpacked(values: &[i32]) -> Vec<u8> {
        let mut buf = Vec::new();
        for &v in values {
            buf.write_tag(tag(1, WireType::Varint)).unwrap();
            buf.write_int32(v).unwrap();
        }
        buf
    }

    // Helper: encode a message with repeated int32 field (field 1, packed)
    fn encode_repeated_int32_packed(values: &[i32]) -> Vec<u8> {
        let mut buf = Vec::new();
        // Packed: tag with LengthDelimited wire type, then length, then concatenated varints
        buf.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
        let mut packed = Vec::new();
        for &v in values {
            packed.write_int32(v).unwrap();
        }
        buf.write_bytes(&packed).unwrap();
        buf
    }

    // Helper: encode a message with repeated string field (field 2)
    fn encode_repeated_string(values: &[&str]) -> Vec<u8> {
        let mut buf = Vec::new();
        for &s in values {
            buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
            buf.write_string(s).unwrap();
        }
        buf
    }

    // --- FieldIter tests ---

    #[test]
    fn field_iter_repeated_strings() {
        let buf = encode_repeated_string(&["hello", "world", "foo"]);
        let iter = FieldIter::new(&buf, 2, Reader::read_string);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn field_iter_empty_message() {
        let buf = Vec::new();
        let iter = FieldIter::new(&buf, 1, Reader::read_string);
        let values: Vec<_> = iter.collect::<Vec<_>>();
        assert!(values.is_empty());
    }

    #[test]
    fn field_iter_skips_other_fields() {
        let mut buf = Vec::new();
        // Field 1: int32 = 42
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(42).unwrap();
        // Field 2: string = "target"
        buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
        buf.write_string("target").unwrap();
        // Field 3: fixed32 = 99
        buf.write_tag(tag(3, WireType::Fixed32)).unwrap();
        buf.write_fixed32(99).unwrap();
        // Field 2: string = "also target"
        buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
        buf.write_string("also target").unwrap();

        let iter = FieldIter::new(&buf, 2, Reader::read_string);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec!["target", "also target"]);
    }

    // --- PackedFieldIter tests ---

    #[test]
    fn packed_iter_unpacked_values() {
        let buf = encode_repeated_int32_unpacked(&[1, 2, 3]);
        let iter = PackedFieldIter::new(&buf, 1, WireType::Varint, Reader::read_int32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn packed_iter_packed_values() {
        let buf = encode_repeated_int32_packed(&[10, 20, 30]);
        let iter = PackedFieldIter::new(&buf, 1, WireType::Varint, Reader::read_int32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn packed_iter_mixed_packed_and_unpacked() {
        let mut buf = Vec::new();
        // Unpacked: field 1 = 1
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(1).unwrap();
        // Packed: field 1 = [2, 3]
        buf.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
        let mut packed = Vec::new();
        packed.write_int32(2).unwrap();
        packed.write_int32(3).unwrap();
        buf.write_bytes(&packed).unwrap();
        // Unpacked: field 1 = 4
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(4).unwrap();

        let iter = PackedFieldIter::new(&buf, 1, WireType::Varint, Reader::read_int32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![1, 2, 3, 4]);
    }

    #[test]
    fn packed_iter_skips_other_fields() {
        let mut buf = Vec::new();
        // Field 2: string = "noise"
        buf.write_tag(tag(2, WireType::LengthDelimited)).unwrap();
        buf.write_string("noise").unwrap();
        // Field 1: int32 = 42
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(42).unwrap();
        // Field 3: fixed64 = 99
        buf.write_tag(tag(3, WireType::Fixed64)).unwrap();
        buf.write_fixed64(99).unwrap();

        let iter = PackedFieldIter::new(&buf, 1, WireType::Varint, Reader::read_int32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![42]);
    }

    #[test]
    fn packed_iter_empty_packed_block() {
        let mut buf = Vec::new();
        // Empty packed block for field 1
        buf.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
        buf.write_bytes(&[]).unwrap();
        // Then a normal value
        buf.write_tag(tag(1, WireType::Varint)).unwrap();
        buf.write_int32(99).unwrap();

        let iter = PackedFieldIter::new(&buf, 1, WireType::Varint, Reader::read_int32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![99]);
    }

    #[test]
    fn packed_iter_fixed32() {
        let mut buf = Vec::new();
        // Packed fixed32 values
        buf.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
        let mut packed = Vec::new();
        packed.write_fixed32(100).unwrap();
        packed.write_fixed32(200).unwrap();
        buf.write_bytes(&packed).unwrap();

        let iter = PackedFieldIter::new(&buf, 1, WireType::Fixed32, Reader::read_fixed32);
        let values: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values, vec![100, 200]);
    }

    // --- MapIter tests ---

    fn encode_map_string_int32(entries: &[(&str, i32)]) -> Vec<u8> {
        let mut buf = Vec::new();
        for &(key, value) in entries {
            buf.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
            // Each map entry is a sub-message with key=field1, value=field2
            let mut entry = Vec::new();
            entry.write_tag(tag(1, WireType::LengthDelimited)).unwrap();
            entry.write_string(key).unwrap();
            entry.write_tag(tag(2, WireType::Varint)).unwrap();
            entry.write_int32(value).unwrap();
            buf.write_bytes(&entry).unwrap();
        }
        buf
    }

    fn decode_string_int32_entry<'a>(
        reader: &mut Reader<'a>,
    ) -> Result<(&'a str, i32), DecodeError> {
        let entry_bytes = reader.read_length_delimited()?;
        let mut r = Reader::new(entry_bytes);
        let mut key: &str = "";
        let mut value: i32 = 0;
        while !r.is_empty() {
            let (fn_, wt) = r.read_tag()?;
            match (fn_, wt) {
                (1, WireType::LengthDelimited) => {
                    key = r.read_string()?;
                }
                (2, WireType::Varint) => {
                    value = r.read_int32()?;
                }
                _ => {
                    r.skip_field(wt)?;
                }
            }
        }
        Ok((key, value))
    }

    #[test]
    fn map_iter_basic() {
        let buf = encode_map_string_int32(&[("a", 1), ("b", 2), ("c", 3)]);
        let iter = MapIter::new(&buf, 1, decode_string_int32_entry);
        let entries: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(entries, vec![("a", 1), ("b", 2), ("c", 3)]);
    }

    #[test]
    fn map_iter_empty() {
        let buf = Vec::new();
        let iter = MapIter::new(&buf, 1, decode_string_int32_entry);
        let entries: Vec<_> = iter.collect::<Vec<_>>();
        assert!(entries.is_empty());
    }
}
