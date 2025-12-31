//! General builder types for working with certain field types.

use std::marker::PhantomData;

use crate::{
    ProtoResult, ScratchBuffer, ScratchWriter, Writer,
    helpers::*,
    types::{MessageBuilder, ProtobufValue, WireType},
};

/// A generic map builder.
pub struct GenericMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: MapScalar + ?Sized,
    V: MapScalar + ?Sized,
{
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<K>,
    _value_type: PhantomData<V>,
}

impl<'w, S, K, V> GenericMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: MapScalar + ?Sized,
    V: MapScalar + ?Sized,
{
    /// Creates a new `GenericMapBuilder` with the given field tag and scratch writer.
    pub fn new(field_tag: u32, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag,
            writer,
            _key_type: PhantomData,
            _value_type: PhantomData,
        }
    }

    /// Writes an entry to the map.
    pub fn write_entry<K2, V2>(&mut self, key: K2, value: V2) -> ProtoResult<()>
    where
        K2: AsRef<K>,
        V2: AsRef<V>,
    {
        self.writer.write_tag(self.field_tag)?;

        let kv_len = (key.as_ref().write_size() + value.as_ref().write_size()) as u64;
        self.writer.write_varint(kv_len)?;

        key.as_ref().write_scalar(1, self.writer)?;
        value.as_ref().write_scalar(2, self.writer)
    }
}

/// A scalar value suitable as a map key or value.
pub trait MapScalar {
    /// Returns the size of the scalar value, in bytes, when serialized.
    fn write_size(&self) -> usize;

    /// Writes the scalar value to the writer with the given field number.
    ///
    /// # Errors
    ///
    /// If there is an error writing the scalar, an error is returned.
    fn write_scalar<W: Writer>(&self, field_number: u32, writer: &mut W) -> ProtoResult<()>;
}

macro_rules! map_scalar_impl {
    (deref, from => $ty:ty, to => $scaled_ty:ty, $sizeof_fn:ident, $write_fn:ident) => {
        impl MapScalar for $ty {
            fn write_size(&self) -> usize {
                $sizeof_fn(<$scaled_ty>::from(*self))
            }

            fn write_scalar<W: Writer>(
                &self,
                field_number: u32,
                writer: &mut W,
            ) -> ProtoResult<()> {
                writer.write_with_tag(tag(field_number, WireType::Varint), |w| {
                    w.$write_fn(<$scaled_ty>::from(*self))
                })
            }
        }
    };

    (deref, from => $ty:ty, $sizeof_fn:ident, $write_fn:ident) => {
        impl MapScalar for $ty {
            fn write_size(&self) -> usize {
                $sizeof_fn(*self)
            }

            fn write_scalar<W: Writer>(
                &self,
                field_number: u32,
                writer: &mut W,
            ) -> ProtoResult<()> {
                writer.write_with_tag(tag(field_number, WireType::Varint), |w| w.$write_fn(*self))
            }
        }
    };

    (from => $ty:ty, $sizeof_fn:ident, $write_fn:ident) => {
        impl MapScalar for $ty {
            fn write_size(&self) -> usize {
                $sizeof_fn(self)
            }

            fn write_scalar<W: Writer>(
                &self,
                field_number: u32,
                writer: &mut W,
            ) -> ProtoResult<()> {
                writer.write_with_tag(tag(field_number, WireType::Varint), |w| w.$write_fn(self))
            }
        }
    };
}

map_scalar_impl!(deref, from => u8, to => u32, sizeof_uint32, write_uint32);
map_scalar_impl!(deref, from => u16, to => u32, sizeof_uint32, write_uint32);
map_scalar_impl!(deref, from => u32, to => u32, sizeof_uint32, write_uint32);
map_scalar_impl!(deref, from => u64, to => u64, sizeof_uint64, write_uint64);
map_scalar_impl!(deref, from => i8, to => i32, sizeof_sint32, write_sint32);
map_scalar_impl!(deref, from => i16, to => i32, sizeof_sint32, write_sint32);
map_scalar_impl!(deref, from => i32, to => i32, sizeof_sint32, write_sint32);
map_scalar_impl!(deref, from => i64, to => i64, sizeof_sint64, write_sint64);
map_scalar_impl!(deref, from => f32, sizeof_f32, write_float);
map_scalar_impl!(deref, from => f64, sizeof_f64, write_double);
map_scalar_impl!(deref, from => bool, sizeof_bool, write_bool);
map_scalar_impl!(from => str, sizeof_str, write_string);
map_scalar_impl!(from => [u8], sizeof_bytes, write_bytes);

/// A map builder for maps with message values.
///
/// Similar to [`GenericMapBuilder`], but for message value types.
pub struct MessageMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: MapScalar + ?Sized,
    V: MessageBuilder<S>,
{
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<K>,
    _value_type: PhantomData<V>,
}

impl<'w, S, K, V> MessageMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: MapScalar + ?Sized,
    V: MessageBuilder<S>,
{
    /// Creates a new `MessageMapBuilder` with the given field tag, scratch writer, and factory.
    pub fn new(field_tag: u32, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag,
            writer,
            _key_type: PhantomData,
            _value_type: PhantomData,
        }
    }

    /// Writes an entry to the map using a callback.
    ///
    /// The callback receives a mutable reference to the value builder and should
    /// populate the message fields.
    pub fn write_entry<K2, F>(&mut self, key: K2, f: F) -> ProtoResult<()>
    where
        K2: AsRef<K>,
        F: FnOnce(&mut V::Builder<'_>) -> ProtoResult<()>,
    {
        let key_ref = key.as_ref();
        self.writer.write_tag(self.field_tag)?;
        self.writer.track_message(move |sw| {
            // Map entries are just like a series of repeated messages, where the message
            // has two fields: the key (field 1), and the value (field 2).
            key_ref.write_scalar(1, sw)?;
            sw.write_tag(tag(2, WireType::LengthDelimited))?;
            sw.track_message(move |sw| {
                let mut builder = V::from_writer(sw);
                f(&mut builder)
            })
        })
    }
}

/// A repeated field builder.
pub struct RepeatedBuilder<'w, S, T, V: ?Sized> {
    field_number: u32,
    writer: &'w mut ScratchWriter<S>,
    _value_type: PhantomData<(T, V)>,
}

impl<'w, S, T, V> RepeatedBuilder<'w, S, T, V>
where
    S: ScratchBuffer,
    T: ProtobufValue<V>,
    V: ?Sized,
{
    /// Creates a new `RepeatedBuilder` with the given field number and scratch writer.
    pub fn new(field_number: u32, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_number,
            writer,
            _value_type: PhantomData,
        }
    }

    /// Adds a new value to the repeated field.
    pub fn add(&mut self, value: &V) -> ProtoResult<()> {
        self.writer
            .write_tag(tag(self.field_number, T::wire_type()))?;
        T::write_value(self.writer, value)
    }

    /// Adds new values from an iterator to the repeated field.
    pub fn add_many<I, IT>(&mut self, values: I) -> ProtoResult<()>
    where
        I: IntoIterator<Item = IT>,
        IT: std::borrow::Borrow<V>,
    {
        self.add_many_mapped(values, std::convert::identity)
    }

    /// Adds new values from an iterator to the repeated field after mapping their value.
    pub fn add_many_mapped<'a, I, IT, F, R>(&mut self, values: I, map: F) -> ProtoResult<()>
    where
        I: IntoIterator<Item = IT>,
        IT: 'a,
        F: Fn(IT) -> R,
        R: std::borrow::Borrow<V> + 'a,
    {
        if T::packable() {
            self.writer
                .write_tag(tag(self.field_number, WireType::LengthDelimited))?;
            self.writer.track_message(|writer| {
                for value in values {
                    let value = map(value);
                    T::write_value(writer, value.borrow())?;
                }
                Ok(())
            })
        } else {
            for value in values {
                let value = map(value);
                self.add(value.borrow())?;
            }
            Ok(())
        }
    }
}
