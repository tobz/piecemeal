//! Helper functions and types for working with individual fields.

use crate::types::WireType;

/// Computes the tag value for the given field number and wire type.
#[inline]
pub const fn tag(field_number: u32, wire_type: WireType) -> u32 {
    (field_number << 3) | wire_type.as_u32()
}

/// Computes the total field length of a varint-encoded u64 field.
#[inline]
pub const fn sizeof_varint(v: u64) -> usize {
    match v {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        0x200000..=0xFFFFFFF => 4,
        0x10000000..=0x7FFFFFFFF => 5,
        0x0800000000..=0x3FFFFFFFFFF => 6,
        0x040000000000..=0x1FFFFFFFFFFFF => 7,
        0x02000000000000..=0xFFFFFFFFFFFFFF => 8,
        0x0100000000000000..=0x7FFFFFFFFFFFFFFF => 9,
        _ => 10,
    }
}

/// Computes the total field length for a string field.
///
/// The total field length is equal to the number of bytes in the string plus the number of bytes for the varint-encoded
/// string length.
#[inline]
pub const fn sizeof_str(s: &str) -> usize {
    sizeof_len(s.len())
}

/// Computes the total field length for a bytes field.
///
/// The total field length is equal to the number of bytes in the byte slice plus the number of bytes for the
/// varint-encoded byte slice length.
#[inline]
pub const fn sizeof_bytes(b: &[u8]) -> usize {
    sizeof_len(b.len())
}

/// Computes the total field length for a variable-length chunk of data.
///
/// The total field length is equal to the number of bytes in the chunk plus the number of bytes for the varint-encoded
/// chunk length.
#[inline]
pub const fn sizeof_len(len: usize) -> usize {
    sizeof_varint(len as u64) + len
}

/// Computes the total field length for a varint-encoded i32 field.
#[inline]
pub const fn sizeof_int32(v: i32) -> usize {
    sizeof_varint(v as u64)
}

/// Computes the total field length for a varint-encoded i64 field.
#[inline]
pub const fn sizeof_int64(v: i64) -> usize {
    sizeof_varint(v as u64)
}

/// Computes the total field length for a varint-encoded u32 field.
#[inline]
pub const fn sizeof_uint32(v: u32) -> usize {
    sizeof_varint(v as u64)
}

/// Computes the total field length for a varint-encoded u64 field.
#[inline]
pub const fn sizeof_uint64(v: u64) -> usize {
    sizeof_varint(v)
}

/// Computes the total field length for a fixed-size i32 field.
#[inline]
pub const fn sizeof_sint32(v: i32) -> usize {
    sizeof_varint(((v << 1) ^ (v >> 31)) as u64)
}

/// Computes the total field length for a fixed-size i64 field.
#[inline]
pub const fn sizeof_sint64(v: i64) -> usize {
    sizeof_varint(((v << 1) ^ (v >> 63)) as u64)
}

/// Computes the total field length of a varint-encoded boolean field.
///
/// The size is always 1.
#[inline]
pub const fn sizeof_bool(_: bool) -> usize {
    1
}

/// Computes the total field length of a fixed-size f32 field.
///
/// The size is always 4.
#[inline]
pub const fn sizeof_f32(_: f32) -> usize {
    4
}

/// Computes the total field length of a fixed-size f64 field.
///
/// The size is always 8.
#[inline]
pub const fn sizeof_f64(_: f64) -> usize {
    8
}

/// Computes the total field length of a varint-encoded enum field.
#[inline]
pub const fn sizeof_enum(v: i32) -> usize {
    sizeof_int32(v)
}
