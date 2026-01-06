//! Common types and traits.

use super::{ProtoResult, ScratchWriter, Writer, helpers::tag};

mod private {
    pub trait Sealed {}
}

/// Automatically generated implementations of core traits for Protocol Buffers types.
pub mod protobuf {
    use std::marker::PhantomData;

    use super::{ProtobufType, WireType};

    macro_rules! generate_protobuf_primitive_types {
        ($proto_ty:ident < $field_ty:ty >, $from_ty:ty, $write_fn:ident $(,)?) => {
            impl $crate::types::ProtobufValue<$from_ty> for $proto_ty<$field_ty>
            where
                $field_ty: std::convert::From<$from_ty>,
            {
                fn write_value<W: $crate::Writer>(
                    writer: &mut W,
                    value: &$from_ty,
                ) -> $crate::ProtoResult<()> {
                    writer.$write_fn(<$field_ty>::from(*value))
                }
            }
        };
    }

    macro_rules! generate_protobuf_ref_types {
        ($proto_ty:ident, $from_ty:ty, $write_fn:ident $(,)?) => {
            impl $crate::types::ProtobufValue<$from_ty> for $proto_ty {
                fn write_value<W: $crate::Writer>(
                    writer: &mut W,
                    value: &$from_ty,
                ) -> $crate::ProtoResult<()> {
                    writer.$write_fn(value)
                }
            }

            impl<'a> $crate::types::ProtobufValue<&'a $from_ty> for $proto_ty {
                fn write_value<W: $crate::Writer>(
                    writer: &mut W,
                    value: &&'a $from_ty,
                ) -> $crate::ProtoResult<()> {
                    writer.$write_fn(value)
                }
            }
        };
    }

    /// A variable-length encoded unsigned 64-bit integer.
    ///
    /// This field type can occupy between one and ten bytes on the wire.
    pub struct Varint<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Varint<T> {
        fn wire_type() -> WireType {
            WireType::Varint
        }

        fn packable() -> bool {
            true
        }
    }

    /// A variable-length encoded signed 32-bit integer.
    ///
    /// This field type can occupy between one and five bytes on the wire.
    pub struct Sint32<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Sint32<T> {
        fn wire_type() -> WireType {
            WireType::Varint
        }

        fn packable() -> bool {
            true
        }
    }

    /// A variable-length encoded signed 64-bit integer.
    ///
    /// This field type can occupy between one and ten bytes on the wire.
    pub struct Sint64<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Sint64<T> {
        fn wire_type() -> WireType {
            WireType::Varint
        }

        fn packable() -> bool {
            true
        }
    }

    /// A fixed-length unsigned 32-bit integer.
    ///
    /// This field type always occupies four bytes on the wire.
    pub struct Fixed32<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Fixed32<T> {
        fn wire_type() -> WireType {
            WireType::Fixed32
        }

        fn packable() -> bool {
            true
        }
    }

    /// A fixed-length unsigned 64-bit integer.
    ///
    /// This field type always occupies eight bytes on the wire.
    pub struct Fixed64<T> {
        _bound: PhantomData<T>,
    }

    impl<T> super::ProtobufType for Fixed64<T> {
        fn wire_type() -> WireType {
            WireType::Fixed64
        }

        fn packable() -> bool {
            true
        }
    }

    /// A fixed-length signed 32-bit integer.
    ///
    /// This field type always occupies four bytes on the wire.
    pub struct Sfixed32<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Sfixed32<T> {
        fn wire_type() -> WireType {
            WireType::Fixed32
        }

        fn packable() -> bool {
            true
        }
    }

    /// A fixed-length signed 64-bit integer.
    ///
    /// This field type always occupies eight bytes on the wire.
    pub struct Sfixed64<T> {
        _bound: PhantomData<T>,
    }

    impl<T> ProtobufType for Sfixed64<T> {
        fn wire_type() -> WireType {
            WireType::Fixed64
        }

        fn packable() -> bool {
            true
        }
    }

    /// A variable-length chunk of bytes.
    pub struct Bytes;

    impl ProtobufType for Bytes {
        fn wire_type() -> WireType {
            WireType::LengthDelimited
        }
    }

    // Sealed trait implementations for all wire types.
    impl<T> super::private::Sealed for Varint<T> {}
    impl<T> super::private::Sealed for Sint32<T> {}
    impl<T> super::private::Sealed for Sint64<T> {}
    impl<T> super::private::Sealed for Fixed32<T> {}
    impl<T> super::private::Sealed for Fixed64<T> {}
    impl<T> super::private::Sealed for Sfixed32<T> {}
    impl<T> super::private::Sealed for Sfixed64<T> {}
    impl super::private::Sealed for Bytes {}

    // Scalars: booleans and floating-point numbers.
    generate_protobuf_primitive_types!(Varint<bool>, bool, write_bool);
    generate_protobuf_primitive_types!(Sfixed32<f32>, f32, write_float);
    generate_protobuf_primitive_types!(Sfixed32<f32>, i8, write_float);
    generate_protobuf_primitive_types!(Sfixed32<f32>, i16, write_float);
    generate_protobuf_primitive_types!(Sfixed32<f32>, u8, write_float);
    generate_protobuf_primitive_types!(Sfixed32<f32>, u16, write_float);
    generate_protobuf_primitive_types!(Sfixed64<f64>, f32, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, f64, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, i8, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, i16, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, i32, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, u8, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, u16, write_double);
    generate_protobuf_primitive_types!(Sfixed64<f64>, u32, write_double);

    // Scalars: variable-width integers (signed and unsigned).
    generate_protobuf_primitive_types!(Varint<u32>, u8, write_uint32);
    generate_protobuf_primitive_types!(Varint<u32>, u16, write_uint32);
    generate_protobuf_primitive_types!(Varint<u32>, u32, write_uint32);
    generate_protobuf_primitive_types!(Varint<u64>, u8, write_uint64);
    generate_protobuf_primitive_types!(Varint<u64>, u16, write_uint64);
    generate_protobuf_primitive_types!(Varint<u64>, u32, write_uint64);
    generate_protobuf_primitive_types!(Varint<u64>, u64, write_uint64);
    generate_protobuf_primitive_types!(Varint<i32>, i8, write_int32);
    generate_protobuf_primitive_types!(Varint<i32>, i16, write_int32);
    generate_protobuf_primitive_types!(Varint<i32>, i32, write_int32);
    generate_protobuf_primitive_types!(Varint<i64>, i8, write_int64);
    generate_protobuf_primitive_types!(Varint<i64>, i16, write_int64);
    generate_protobuf_primitive_types!(Varint<i64>, i32, write_int64);
    generate_protobuf_primitive_types!(Varint<i64>, i64, write_int64);
    generate_protobuf_primitive_types!(Sint32<i32>, i8, write_sint32);
    generate_protobuf_primitive_types!(Sint32<i32>, i16, write_sint32);
    generate_protobuf_primitive_types!(Sint32<i32>, i32, write_sint32);
    generate_protobuf_primitive_types!(Sint64<i64>, i8, write_sint64);
    generate_protobuf_primitive_types!(Sint64<i64>, i16, write_sint64);
    generate_protobuf_primitive_types!(Sint64<i64>, i32, write_sint64);
    generate_protobuf_primitive_types!(Sint64<i64>, i64, write_sint64);

    // Scalars: fixed-width integers (signed and unsigned).
    generate_protobuf_primitive_types!(Fixed32<u32>, u8, write_fixed32);
    generate_protobuf_primitive_types!(Fixed32<u32>, u16, write_fixed32);
    generate_protobuf_primitive_types!(Fixed32<u32>, u32, write_fixed32);
    generate_protobuf_primitive_types!(Fixed64<u64>, u8, write_fixed64);
    generate_protobuf_primitive_types!(Fixed64<u64>, u16, write_fixed64);
    generate_protobuf_primitive_types!(Fixed64<u64>, u32, write_fixed64);
    generate_protobuf_primitive_types!(Fixed64<u64>, u64, write_fixed64);
    generate_protobuf_primitive_types!(Sfixed32<i32>, i8, write_sfixed32);
    generate_protobuf_primitive_types!(Sfixed32<i32>, i16, write_sfixed32);
    generate_protobuf_primitive_types!(Sfixed32<i32>, i32, write_sfixed32);
    generate_protobuf_primitive_types!(Sfixed64<i64>, i8, write_sfixed64);
    generate_protobuf_primitive_types!(Sfixed64<i64>, i16, write_sfixed64);
    generate_protobuf_primitive_types!(Sfixed64<i64>, i32, write_sfixed64);
    generate_protobuf_primitive_types!(Sfixed64<i64>, i64, write_sfixed64);

    // Length-delimited types: strings and bytes.
    generate_protobuf_ref_types!(Bytes, str, write_string);
    generate_protobuf_ref_types!(Bytes, [u8], write_bytes);
}

/// Wire type.
pub enum WireType {
    /// Variable-width integer.
    ///
    /// Encodes integers using a variable number of bytes, depending on the magnitude of the value,
    /// consuming between one and ten bytes on the wire.
    ///
    /// See https://protobuf.dev/programming-guides/encoding/#varints for more information.
    Varint,

    /// Fixed 32-bit numerical value: (un)signed integer or single-precision floating point number.
    ///
    /// Consumes four bytes (32-bit) on the wire.
    Fixed32,

    /// Fixed 64-bit numerical value: (un)signed integer or double-precision floating point number.
    ///
    /// Consumes eight bytes (64-bit) on the wire.
    Fixed64,

    /// Length-delimiter field.
    ///
    /// Used for fields with variable length, such as strings, bytes, embedded messages, and packed
    /// repeated fields.
    LengthDelimited,
}

impl WireType {
    /// Gets the integer representation of the wire type.
    pub const fn as_u32(&self) -> u32 {
        match self {
            WireType::Varint => 0,
            WireType::Fixed64 => 1,
            WireType::LengthDelimited => 2,
            WireType::Fixed32 => 5,
        }
    }
}

/// A Protocol Buffers type.
///
/// This trait is implemented by the integral wire types to describe how they are encoded on the wire.
pub trait ProtobufType: private::Sealed {
    /// Returns the wire type for this type.
    fn wire_type() -> WireType;

    /// Returns `true` if this type can be packed.
    fn packable() -> bool {
        false
    }
}

/// A Protocol Buffers value.
pub trait ProtobufValue<T: ?Sized>: ProtobufType {
    /// Writes the value to the given writer.
    ///
    /// # Errors
    ///
    /// If the value cannot be written to the writer, an error is returned.
    fn write_value<W: Writer>(writer: &mut W, value: &T) -> ProtoResult<()>;

    /// Writes the value as a complete field to the given writer.
    ///
    /// # Errors
    fn write_field<W: Writer>(writer: &mut W, field_number: u32, value: &T) -> ProtoResult<()> {
        writer.write_tag(tag(field_number, Self::wire_type()))?;
        Self::write_value(writer, value)
    }
}

/// A marker trait for types that can be used as map keys.
///
/// In Protocol Buffers, map keys can be any scalar type besides floating-point numbers and bytes.
pub trait MapKey: private::Sealed {}

impl private::Sealed for bool {}
impl private::Sealed for i8 {}
impl private::Sealed for i16 {}
impl private::Sealed for i32 {}
impl private::Sealed for i64 {}
impl private::Sealed for u8 {}
impl private::Sealed for u16 {}
impl private::Sealed for u32 {}
impl private::Sealed for u64 {}
impl private::Sealed for str {}
impl<T: private::Sealed + ?Sized> private::Sealed for &T {}

impl MapKey for bool {}
impl MapKey for i8 {}
impl MapKey for i16 {}
impl MapKey for i32 {}
impl MapKey for i64 {}
impl MapKey for u8 {}
impl MapKey for u16 {}
impl MapKey for u32 {}
impl MapKey for u64 {}
impl MapKey for str {}
impl MapKey for &str {}

/// Message builder base trait.
pub trait MessageBuilderBase<S> {
    /// Message builder type.
    type Builder<'a>
    where
        S: 'a;
}

/// A message builder.
pub trait MessageBuilder<S>: MessageBuilderBase<S> {
    /// Create a new message builder from the given writer.
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w>;
}
