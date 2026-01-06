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
    // Each varint byte encodes 7 bits, so we essentially calculate how many 7-bit groups we need.
    let bits = 64 - v.leading_zeros() as usize;
    bits.saturating_sub(1) / 7 + 1
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

/// Computes the total field length for a zigzag-encoded i32 field.
#[inline]
pub const fn sizeof_sint32(v: i32) -> usize {
    sizeof_varint(((v << 1) ^ (v >> 31)) as u32 as u64)
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

#[cfg(test)]
mod tests {
    use super::*;

    // tag() tests
    #[test]
    fn test_tag_varint() {
        // tag = (field_number << 3) | wire_type
        assert_eq!(tag(1, WireType::Varint), 0x08); // (1 << 3) | 0 = 8
        assert_eq!(tag(2, WireType::Varint), 0x10); // (2 << 3) | 0 = 16
        assert_eq!(tag(15, WireType::Varint), 0x78); // (15 << 3) | 0 = 120
        assert_eq!(tag(16, WireType::Varint), 0x80); // (16 << 3) | 0 = 128
    }

    #[test]
    fn test_tag_fixed64() {
        assert_eq!(tag(1, WireType::Fixed64), 0x09); // (1 << 3) | 1 = 9
        assert_eq!(tag(2, WireType::Fixed64), 0x11); // (2 << 3) | 1 = 17
    }

    #[test]
    fn test_tag_length_delimited() {
        assert_eq!(tag(1, WireType::LengthDelimited), 0x0a); // (1 << 3) | 2 = 10
        assert_eq!(tag(2, WireType::LengthDelimited), 0x12); // (2 << 3) | 2 = 18
    }

    #[test]
    fn test_tag_fixed32() {
        assert_eq!(tag(1, WireType::Fixed32), 0x0d); // (1 << 3) | 5 = 13
        assert_eq!(tag(2, WireType::Fixed32), 0x15); // (2 << 3) | 5 = 21
    }

    // sizeof_varint() tests - boundary conditions at 7-bit intervals
    #[test]
    fn test_sizeof_varint_1_byte() {
        assert_eq!(sizeof_varint(0), 1);
        assert_eq!(sizeof_varint(1), 1);
        assert_eq!(sizeof_varint(0x7F), 1); // 127
    }

    #[test]
    fn test_sizeof_varint_2_bytes() {
        assert_eq!(sizeof_varint(0x80), 2); // 128
        assert_eq!(sizeof_varint(0x3FFF), 2); // 16383
    }

    #[test]
    fn test_sizeof_varint_3_bytes() {
        assert_eq!(sizeof_varint(0x4000), 3); // 16384
        assert_eq!(sizeof_varint(0x1FFFFF), 3); // 2097151
    }

    #[test]
    fn test_sizeof_varint_4_bytes() {
        assert_eq!(sizeof_varint(0x200000), 4); // 2097152
        assert_eq!(sizeof_varint(0xFFFFFFF), 4); // 268435455
    }

    #[test]
    fn test_sizeof_varint_5_bytes() {
        assert_eq!(sizeof_varint(0x10000000), 5); // 268435456
        assert_eq!(sizeof_varint(0x7FFFFFFFF), 5); // 34359738367
    }

    #[test]
    fn test_sizeof_varint_6_bytes() {
        assert_eq!(sizeof_varint(0x0800000000), 6);
        assert_eq!(sizeof_varint(0x3FFFFFFFFFF), 6);
    }

    #[test]
    fn test_sizeof_varint_7_bytes() {
        assert_eq!(sizeof_varint(0x040000000000), 7);
        assert_eq!(sizeof_varint(0x1FFFFFFFFFFFF), 7);
    }

    #[test]
    fn test_sizeof_varint_8_bytes() {
        assert_eq!(sizeof_varint(0x02000000000000), 8);
        assert_eq!(sizeof_varint(0xFFFFFFFFFFFFFF), 8);
    }

    #[test]
    fn test_sizeof_varint_9_bytes() {
        assert_eq!(sizeof_varint(0x0100000000000000), 9);
        assert_eq!(sizeof_varint(0x7FFFFFFFFFFFFFFF), 9);
    }

    #[test]
    fn test_sizeof_varint_10_bytes() {
        assert_eq!(sizeof_varint(0x8000000000000000), 10);
        assert_eq!(sizeof_varint(u64::MAX), 10);
    }

    // sizeof_str() and sizeof_bytes() tests
    #[test]
    fn test_sizeof_str() {
        assert_eq!(sizeof_str(""), 1); // 0 bytes + 1 for length
        assert_eq!(sizeof_str("a"), 2); // 1 byte + 1 for length
        assert_eq!(sizeof_str("hello"), 6); // 5 bytes + 1 for length
    }

    #[test]
    fn test_sizeof_str_length_boundary() {
        // String with 127 chars (length fits in 1 byte varint)
        let s127 = "a".repeat(127);
        assert_eq!(sizeof_str(&s127), 128); // 127 bytes + 1 for length

        // String with 128 chars (length needs 2 bytes varint)
        let s128 = "a".repeat(128);
        assert_eq!(sizeof_str(&s128), 130); // 128 bytes + 2 for length
    }

    #[test]
    fn test_sizeof_bytes() {
        assert_eq!(sizeof_bytes(&[]), 1);
        assert_eq!(sizeof_bytes(&[1, 2, 3]), 4);
    }

    #[test]
    fn test_sizeof_bytes_length_boundary() {
        assert_eq!(sizeof_bytes(&[0u8; 127]), 128); // 127 + 1
        assert_eq!(sizeof_bytes(&[0u8; 128]), 130); // 128 + 2
    }

    #[test]
    fn test_sizeof_len() {
        assert_eq!(sizeof_len(0), 1); // 0 + 1 for length
        assert_eq!(sizeof_len(127), 128); // 127 + 1
        assert_eq!(sizeof_len(128), 130); // 128 + 2
    }

    // sizeof_int32() tests - negative numbers are sign-extended to 64 bits
    #[test]
    fn test_sizeof_int32() {
        assert_eq!(sizeof_int32(0), 1);
        assert_eq!(sizeof_int32(1), 1);
        assert_eq!(sizeof_int32(127), 1);
        assert_eq!(sizeof_int32(128), 2);
    }

    #[test]
    fn test_sizeof_int32_negative() {
        // Negative values are sign-extended to 64 bits, so they take 10 bytes
        assert_eq!(sizeof_int32(-1), 10);
        assert_eq!(sizeof_int32(i32::MIN), 10);
    }

    #[test]
    fn test_sizeof_int32_max() {
        assert_eq!(sizeof_int32(i32::MAX), 5);
    }

    // sizeof_int64() tests
    #[test]
    fn test_sizeof_int64() {
        assert_eq!(sizeof_int64(0), 1);
        assert_eq!(sizeof_int64(1), 1);
        assert_eq!(sizeof_int64(-1), 10);
        assert_eq!(sizeof_int64(i64::MAX), 9);
        assert_eq!(sizeof_int64(i64::MIN), 10);
    }

    // sizeof_uint32() tests
    #[test]
    fn test_sizeof_uint32() {
        assert_eq!(sizeof_uint32(0), 1);
        assert_eq!(sizeof_uint32(127), 1);
        assert_eq!(sizeof_uint32(128), 2);
        assert_eq!(sizeof_uint32(u32::MAX), 5);
    }

    // sizeof_uint64() tests
    #[test]
    fn test_sizeof_uint64() {
        assert_eq!(sizeof_uint64(0), 1);
        assert_eq!(sizeof_uint64(127), 1);
        assert_eq!(sizeof_uint64(128), 2);
        assert_eq!(sizeof_uint64(u64::MAX), 10);
    }

    // sizeof_sint32() tests - zigzag encoding
    #[test]
    fn test_sizeof_sint32() {
        // ZigZag encoding: (n << 1) ^ (n >> 31)
        // zigzag(0) = 0, zigzag(-1) = 1, zigzag(1) = 2, zigzag(-2) = 3
        assert_eq!(sizeof_sint32(0), 1);
        assert_eq!(sizeof_sint32(-1), 1);
        assert_eq!(sizeof_sint32(1), 1);
        assert_eq!(sizeof_sint32(-2), 1);
        assert_eq!(sizeof_sint32(63), 1);
        assert_eq!(sizeof_sint32(-64), 1);
    }

    #[test]
    fn test_sizeof_sint32_extremes() {
        // Zigzag encoding maps i32::MAX to u32::MAX - 1 (0xFFFFFFFE) = 5 bytes
        // Zigzag encoding maps i32::MIN to u32::MAX (0xFFFFFFFF) = 5 bytes
        assert_eq!(sizeof_sint32(i32::MAX), 5);
        assert_eq!(sizeof_sint32(i32::MIN), 5);
    }

    // sizeof_sint64() tests
    #[test]
    fn test_sizeof_sint64() {
        assert_eq!(sizeof_sint64(0), 1);
        assert_eq!(sizeof_sint64(-1), 1);
        assert_eq!(sizeof_sint64(1), 1);
        assert_eq!(sizeof_sint64(-2), 1);
    }

    #[test]
    fn test_sizeof_sint64_extremes() {
        assert_eq!(sizeof_sint64(i64::MAX), 10);
        assert_eq!(sizeof_sint64(i64::MIN), 10);
    }

    // Fixed size tests
    #[test]
    fn test_sizeof_bool() {
        assert_eq!(sizeof_bool(true), 1);
        assert_eq!(sizeof_bool(false), 1);
    }

    #[test]
    fn test_sizeof_f32() {
        assert_eq!(sizeof_f32(0.0), 4);
        assert_eq!(sizeof_f32(f32::MAX), 4);
        assert_eq!(sizeof_f32(f32::MIN), 4);
        assert_eq!(sizeof_f32(f32::NAN), 4);
    }

    #[test]
    fn test_sizeof_f64() {
        assert_eq!(sizeof_f64(0.0), 8);
        assert_eq!(sizeof_f64(f64::MAX), 8);
        assert_eq!(sizeof_f64(f64::MIN), 8);
        assert_eq!(sizeof_f64(f64::NAN), 8);
    }

    // sizeof_enum() tests
    #[test]
    fn test_sizeof_enum() {
        assert_eq!(sizeof_enum(0), 1);
        assert_eq!(sizeof_enum(127), 1);
        assert_eq!(sizeof_enum(128), 2);
        assert_eq!(sizeof_enum(-1), 10); // Same as int32
    }
}
