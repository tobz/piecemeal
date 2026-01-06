//! General builder types for working with certain field types.

use std::{io, marker::PhantomData};

use crate::{
    ScratchBuffer, ScratchWriter, Writer,
    helpers::*,
    types::{MapKey, MessageBuilder, ProtobufType, ProtobufValue, WireType},
};

/// A generic map builder.
pub struct GenericMapBuilder<'w, S, K, V> {
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<K>,
    _value_type: PhantomData<V>,
}

impl<'w, S, K, V> GenericMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: ProtobufType,
    V: ProtobufType,
{
    /// Creates a new `GenericMapBuilder` with the given field number and scratch writer.
    pub fn new(field_number: u32, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag: tag(field_number, WireType::LengthDelimited),
            writer,
            _key_type: PhantomData,
            _value_type: PhantomData,
        }
    }

    /// Writes an entry to the map.
    pub fn write_entry<K2, V2>(&mut self, key: K2, value: V2) -> io::Result<()>
    where
        K: ProtobufValue<K2>,
        K2: MapKey,
        V: ProtobufValue<V2>,
    {
        self.writer.write_tag(self.field_tag)?;
        self.writer.track_message(|sw| {
            K::write_field(sw, 1, &key)?;
            V::write_field(sw, 2, &value)
        })
    }
}

/// A map builder for maps with message values.
///
/// Similar to [`GenericMapBuilder`], but for message value types.
pub struct MessageMapBuilder<'w, S, K, V> {
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<K>,
    _value_type: PhantomData<V>,
}

impl<'w, S, K, V> MessageMapBuilder<'w, S, K, V>
where
    S: ScratchBuffer,
    K: ProtobufType,
    V: MessageBuilder<S>,
{
    /// Creates a new `MessageMapBuilder` with the given field number and scratch writer.
    pub fn new(field_number: u32, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag: tag(field_number, WireType::LengthDelimited),
            writer,
            _key_type: PhantomData,
            _value_type: PhantomData,
        }
    }

    /// Writes an entry to the map using a callback.
    ///
    /// The callback receives a mutable reference to the value builder and should
    /// populate the message fields.
    pub fn write_entry<K2, F>(&mut self, key: K2, f: F) -> io::Result<()>
    where
        K: ProtobufValue<K2>,
        K2: MapKey,
        F: FnOnce(&mut V::Builder<'_>) -> io::Result<()>,
    {
        self.writer.write_tag(self.field_tag)?;
        self.writer.track_message(move |sw| {
            // Map entries are just like a series of repeated messages, where the message
            // has two fields: the key (field 1), and the value (field 2).
            K::write_field(sw, 1, &key)?;

            sw.write_tag(tag(2, WireType::LengthDelimited))?;
            sw.track_message(move |sw| {
                let mut builder = V::from_writer(sw);
                f(&mut builder)
            })
        })
    }
}

/// A repeated field builder.
pub struct RepeatedBuilder<'w, S, T> {
    field_tag: u32,
    packed_field_tag: u32,
    can_pack: bool,
    writer: &'w mut ScratchWriter<S>,
    _value_type: PhantomData<T>,
}

impl<'w, S, T> RepeatedBuilder<'w, S, T>
where
    S: ScratchBuffer,
    T: ProtobufType,
{
    /// Creates a new `RepeatedBuilder` with the given field number and scratch writer.
    pub fn new(field_number: u32, can_pack: bool, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag: tag(field_number, T::wire_type()),
            packed_field_tag: tag(field_number, WireType::LengthDelimited),
            can_pack,
            writer,
            _value_type: PhantomData,
        }
    }

    /// Adds a new value to the repeated field.
    pub fn add<V>(&mut self, value: V) -> io::Result<()>
    where
        T: ProtobufValue<V>,
    {
        self.writer.write_tag(self.field_tag)?;

        T::write_value(self.writer, &value)
    }

    /// Adds new values from an iterator to the repeated field.
    pub fn add_many<I, IT>(&mut self, values: I) -> io::Result<()>
    where
        I: IntoIterator<Item = IT>,
        T: ProtobufValue<IT>,
    {
        self.add_many_mapped(values, std::convert::identity)
    }

    /// Adds new values from an iterator to the repeated field after mapping their value.
    pub fn add_many_mapped<'a, I, IT, F, R>(&mut self, values: I, map: F) -> io::Result<()>
    where
        I: IntoIterator<Item = IT>,
        IT: 'a,
        F: Fn(IT) -> R,
        T: ProtobufValue<R>,
    {
        if T::packable() && self.can_pack {
            self.writer.write_tag(self.packed_field_tag)?;
            self.writer.track_message(|writer| {
                for value in values {
                    let value = map(value);
                    T::write_value(writer, &value)?;
                }
                Ok(())
            })
        } else {
            for value in values {
                let value = map(value);
                self.add(value)?;
            }
            Ok(())
        }
    }
}
