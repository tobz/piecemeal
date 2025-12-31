//! Common types and traits.

use crate::{ProtoResult, ScratchBuffer, ScratchWriter, Writer};

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

/// Wrapper enum over packed fixed data, similar to `Cow`.
#[derive(Clone, Debug)]
pub enum PackedFixed<'a, T: Copy + PartialEq> {
    /// Variant that carries a reference to raw bytes that may or may not be
    /// aligned, representing a packed set of fixed numbers.
    ///
    /// `PackedFixed` methods called on `Borrowed` will use `read_unaligned()`
    /// to interact with the data without copying all bytes to an aligned buffer
    /// in order to avoid delay from that memory allocation. So far, I can't
    /// think of any way to take advantage of the situations when it is
    /// coincidentally aligned.
    Borrowed(&'a [u8]),
    /// Variant that contains an owned vector of numbers.
    ///
    /// An owned collection of `T`.
    Owned(Vec<T>),
}

impl<'a, T: Copy + PartialEq> PackedFixed<'a, T> {
    /// Return the length of the DATA (not the bytes).
    pub fn len(&self) -> usize {
        match self {
            PackedFixed::Borrowed(bytes) => bytes.len() / ::core::mem::size_of::<T>(),
            PackedFixed::Owned(v) => v.len(),
        }
    }

    /// Mutate in place to `Owned` variant. In the case of `Borrowed`, this
    /// performs a bitwise copy of the entire slice.
    pub fn own(&mut self) {
        match self {
            PackedFixed::Borrowed(_) => {
                *self = self.make_owned_variant_from_unaligned_buf();
            }
            PackedFixed::Owned(_) => {} // no-op for PackedFixed::Owned, just like Cow
        }
    }

    /// Get a `Vec<T>` of the internal data, moving `self` in the process. The
    /// reason we move `self` is so that calling this on an `Owned` variant
    /// will not require copying data. `Borrowed` variants will trigger a
    /// bitwise copy.
    ///
    /// It would be really nice if this could instead return `&[T]` without
    /// moving `self`, but we can't do this for the `Borrowed` variant, so
    /// we have no such method on `PackedFixed` as a whole. And anyway, this is
    /// what `at()` on `Borrowed` is for.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            PackedFixed::Borrowed(_) => self.make_vec_from_unaligned_buf(),
            PackedFixed::Owned(v) => v,
        }
    }

    /// Get the element at index `index`.
    ///
    /// Note that `index` refers to the index of the type `T`, and NOT the byte
    /// index. In the case of `Borrowed`, this index is calculated during
    /// runtime, as if the underlying data was already in form `Vec<T>`.
    pub fn at(&self, index: usize) -> T {
        match self {
            PackedFixed::Borrowed(bytes) => {
                let byte_offset = index * core::mem::size_of::<T>();
                if byte_offset >= bytes.len() {
                    panic!("PackedFixed::at(): Index out of range!");
                }

                let mut ptr = bytes.as_ptr();
                unsafe {
                    ptr = ptr.add(byte_offset);
                    (ptr as *const T).read_unaligned()
                }
            }
            PackedFixed::Owned(v) => v[index],
        }
    }

    /// Mutate `self` to `Owned` variant before returning immutable slice
    pub fn to_slice(&mut self) -> &[T] {
        self.own();
        if let PackedFixed::Owned(ref contents) = *self {
            contents
        } else {
            unreachable!();
        }
    }

    /// Mutate `self` to `Owned` variant before returning mutable slice
    pub fn to_mut_slice(&mut self) -> &mut [T] {
        self.own();
        if let PackedFixed::Owned(ref mut contents) = *self {
            contents
        } else {
            unreachable!();
        }
    }

    /// Returns `true` if no data is contained in the enum.
    pub fn is_empty(&self) -> bool {
        match self {
            PackedFixed::Borrowed(bytes) => bytes.is_empty(),
            PackedFixed::Owned(contents) => contents.is_empty(),
        }
    }

    // This method is private and mainly to avoid repetition in code.
    fn make_vec_from_unaligned_buf(&self) -> Vec<T> {
        match &self {
            PackedFixed::Borrowed(bytes) => unsafe {
                let src = bytes.as_ptr();
                let mut buf = Vec::<T>::with_capacity(self.len());
                let dst = buf.as_mut_ptr() as *mut u8;
                ::core::ptr::copy(src, dst, bytes.len()); // careful to use length in bytes here
                buf.set_len(self.len());
                buf
            },
            _ => unreachable!(),
        }
    }

    // This method is private and mainly to avoid repetition in code.
    fn make_owned_variant_from_unaligned_buf(&self) -> Self {
        match &self {
            PackedFixed::Borrowed(_) => PackedFixed::Owned(self.make_vec_from_unaligned_buf()),
            _ => unreachable!(),
        }
    }
}

/// A message builder.
pub trait MessageBuilder {
    /// Creates a new message builder from the given scratch writer.
    fn from_writer<'w, S: ScratchBuffer>(writer: &'w mut ScratchWriter<S>) -> Self;
}
