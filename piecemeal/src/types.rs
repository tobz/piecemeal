//! Common types and traits.

use crate::{ProtoResult, ScratchWriter, Writer};

/// Automatically generated implementations of core traits for Protocol Buffers types.
pub mod protobuf {
    macro_rules! generate_protobuf_primitive_types {
		(proto_ty => $ty_name:ident, wire_type => $wire_type:ident, rust_types => [$($rust_ty:ty),+], write => { func => $write_fn:ident, deref, as_type => $as_ty:ty } $(,)?) => {
			$(
				impl $crate::types::ProtobufValue<$rust_ty> for $ty_name {
					fn wire_type() -> $crate::types::WireType {
						$crate::types::WireType::$wire_type
					}

					fn packable() -> bool {
						true
					}

					fn write_value<W: $crate::Writer>(writer: &mut W, value: &$rust_ty) -> $crate::ProtoResult<()> {
						writer.$write_fn(*value as $as_ty)
					}
				}
			)+
		};
	}

    macro_rules! generate_protobuf_ref_types {
		(proto_ty => $ty_name:ident, wire_type => $wire_type:ident, rust_types => [$($rust_ty:ty),+], write => { func => $write_fn:ident } $(,)?) => {
			$(
				impl $crate::types::ProtobufValue<$rust_ty> for $ty_name {
					fn wire_type() -> $crate::types::WireType {
						$crate::types::WireType::$wire_type
					}

					fn write_value<W: $crate::Writer>(writer: &mut W, value: &$rust_ty) -> $crate::ProtoResult<()> {
						writer.$write_fn(value)
					}
				}
			)+
		};
	}

    /// A variable-length encoded unsigned 64-bit integer.
    ///
    /// This field type can occupy between one and ten bytes on the wire.
    pub struct Varint;

    /// A variable-length encoded signed 32-bit integer.
    ///
    /// This field tye can occupy between one and five bytes on the wire.
    pub struct Sint32;

    /// A variable-length encoded signed 64-bit integer.
    ///
    /// This field type can occupy between one and ten bytes on the wire.
    pub struct Sint64;

    /// A fixed-length unsigned 32-bit integer.
    ///
    /// This field type always occupies four bytes on the wire.
    pub struct Fixed32;

    /// A fixed-length unsigned 64-bit integer.
    ///
    /// This field type always occupies eight bytes on the wire.
    pub struct Fixed64;

    /// A fixed-length signed 32-bit integer.
    ///
    /// This field type always occupies four bytes on the wire.
    pub struct Sfixed32;

    /// A fixed-length signed 64-bit integer.
    ///
    /// This field type always occupies eight bytes on the wire.
    pub struct Sfixed64;

    /// A variable-length chunk of bytes.
    pub struct Bytes;

    generate_protobuf_primitive_types!(
        proto_ty => Varint,
        wire_type => Varint,
        rust_types => [i8, i16, i32, i64, isize, u8, u16, u32, u64, usize],
        write => { func => write_varint, deref, as_type => u64 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sint32,
        wire_type => Varint,
        rust_types => [i8, i16, i32],
        write => { func => write_sint32, deref, as_type => i32 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sint64,
        wire_type => Varint,
        rust_types => [i8, i16, i32, i64, isize],
        write => { func => write_sint64, deref, as_type => i64 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Fixed32,
        wire_type => Fixed32,
        rust_types => [u8, u16, u32],
        write => { func => write_fixed32, deref, as_type => u32 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Fixed64,
        wire_type => Fixed64,
        rust_types => [u8, u16, u32, u64, usize],
        write => { func => write_fixed64, deref, as_type => u64 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sfixed32,
        wire_type => Fixed32,
        rust_types => [i8, i16, i32],
        write => { func => write_sfixed32, deref, as_type => i32 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sfixed32,
        wire_type => Fixed32,
        rust_types => [f32],
        write => { func => write_float, deref, as_type => f32 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sfixed64,
        wire_type => Fixed64,
        rust_types => [i8, i16, i32, i64, isize],
        write => { func => write_sfixed64, deref, as_type => i64 },
    );
    generate_protobuf_primitive_types!(
        proto_ty => Sfixed64,
        wire_type => Fixed64,
        rust_types => [f64],
        write => { func => write_double, deref, as_type => f64 },
    );
    generate_protobuf_ref_types!(
        proto_ty => Bytes,
        wire_type => LengthDelimited,
        rust_types => [str],
        write => { func => write_string },
    );
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

    /// Fixed 64-bit integer or double-precision floating-point number.
    ///
    /// Consumes eight bytes (64-bit) on the wire.
    Fixed64,

    /// Length-delimiter field.
    ///
    /// Used for fields with variable length, such as strings, bytes, embedded messages, and packed
    /// repeated fields.
    LengthDelimited,

    /// Fixed 32-bit integer or single-precision floating-point number.
    ///
    /// Consumes four bytes (32-bit) on the wire.
    Fixed32,
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

/// A non-complex value type, excluding strings and bytes.
///
/// This is effectively all numeric types and booleans.
///
/// Another way to view primitives types is that they all have a bounded size and can potentially be
/// encoded contiguously without the need for a field tag between each value. This property is what
/// allows for packed repeated fields when the field is a primitive type.
pub trait Primitive {}

impl<T> Primitive for T where T: ProtobufValue<T> {}

/// A non-complex value type.
///
/// Scalar values include primitive values as well as strings and bytes.
///
/// Essentially, any value that isn't an object or map is a scalar type.
pub trait Scalar {}

impl<T> Scalar for T where T: ProtobufValue<T> {}

/// A Protocol Buffers value.
pub trait ProtobufValue<T: ?Sized> {
    /// [Wire type][wiretype] of the value.
    ///
    /// [wiretype]: https://protobuf.dev/programming-guides/encoding/#structure
    fn wire_type() -> WireType;

    /// Whether the value can be packed.
    fn packable() -> bool {
        false
    }

    /// Writes the value to the given writer.
    ///
    /// # Errors
    ///
    /// If the value cannot be written to the writer, an error is returned.
    fn write_value<W: Writer>(writer: &mut W, value: &T) -> ProtoResult<()>;
}

macro_rules! impl_basic_traits {
	(primitive => [$($t:ty),+]) => {
		$(
			impl Primitive for $t {}
		)+
	};
	(scalar => [$($t:ty),+]) => {
		$(
			impl Scalar for $t {}
		)+
	};
}

impl_basic_traits!(primitive => [bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64]);
impl_basic_traits!(scalar => [bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64, str, [u8]]);

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
