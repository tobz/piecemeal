//! General builder types for working with certain field types.

use std::{borrow::Borrow, marker::PhantomData};

use crate::{
    ProtoResult, ScratchBuffer, ScratchWriter, Writer,
    helpers::*,
    types::{MessageBuilder, ProtobufValue, WireType},
};

/// A marker trait for types that can be used as map keys.
///
/// In Protocol Buffers, map keys can be any scalar type besides floating-point types and bytes.
pub trait MapKey {}

// TODO: sealed impl for `MapKey`
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
impl MapKey for [u8] {}

/// A generic map builder.
pub struct GenericMapBuilder<'w, S, KP, KR: ?Sized, VP, VR: ?Sized> {
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<(KP, KR)>,
    _value_type: PhantomData<(VP, VR)>,
}

impl<'w, S, KP, KR, VP, VR> GenericMapBuilder<'w, S, KP, KR, VP, VR>
where
    S: ScratchBuffer,
    KP: ProtobufValue<KR>,
    KR: MapKey + ?Sized,
    VP: ProtobufValue<VR>,
    VR: ?Sized,
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
    pub fn write_entry(&mut self, key: &KR, value: &VR) -> ProtoResult<()> {
        self.writer.write_tag(self.field_tag)?;
        self.writer.track_message(|sw| {
            KP::write_field(sw, 1, key)?;
            VP::write_field(sw, 2, value)
        })
    }
}

/// A map builder for maps with message values.
///
/// Similar to [`GenericMapBuilder`], but for message value types.
pub struct MessageMapBuilder<'w, S, KP, KR: ?Sized, V> {
    field_tag: u32,
    writer: &'w mut ScratchWriter<S>,
    _key_type: PhantomData<(KP, KR)>,
    _value_type: PhantomData<V>,
}

impl<'w, S, KP, KR, V> MessageMapBuilder<'w, S, KP, KR, V>
where
    S: ScratchBuffer,
    KP: ProtobufValue<KR>,
    KR: MapKey + ?Sized,
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
    pub fn write_entry<F>(&mut self, key: &KR, f: F) -> ProtoResult<()>
    where
        F: FnOnce(&mut V::Builder<'_>) -> ProtoResult<()>,
    {
        self.writer.write_tag(self.field_tag)?;
        self.writer.track_message(move |sw| {
            // Map entries are just like a series of repeated messages, where the message
            // has two fields: the key (field 1), and the value (field 2).
            KP::write_field(sw, 1, key)?;

            sw.write_tag(tag(2, WireType::LengthDelimited))?;
            sw.track_message(move |sw| {
                let mut builder = V::from_writer(sw);
                f(&mut builder)
            })
        })
    }
}

/// A repeated field builder.
pub struct RepeatedBuilder<'w, S, VP, VR: ?Sized> {
    field_tag: u32,
    packed_field_tag: u32,
    can_pack: bool,
    writer: &'w mut ScratchWriter<S>,
    _value_type: PhantomData<(VP, VR)>,
}

impl<'w, S, VP, VR> RepeatedBuilder<'w, S, VP, VR>
where
    S: ScratchBuffer,
    VP: ProtobufValue<VR>,
    VR: ?Sized,
{
    /// Creates a new `RepeatedBuilder` with the given field number and scratch writer.
    pub fn new(field_number: u32, can_pack: bool, writer: &'w mut ScratchWriter<S>) -> Self {
        Self {
            field_tag: tag(field_number, VP::wire_type()),
            packed_field_tag: tag(field_number, WireType::LengthDelimited),
            can_pack,
            writer,
            _value_type: PhantomData,
        }
    }

    /// Adds a new value to the repeated field.
    pub fn add(&mut self, value: &VR) -> ProtoResult<()> {
        self.writer.write_tag(self.field_tag)?;

        VP::write_value(self.writer, value)
    }

    /// Adds new values from an iterator to the repeated field.
    pub fn add_many<I, IT>(&mut self, values: I) -> ProtoResult<()>
    where
        I: IntoIterator<Item = IT>,
        IT: Borrow<VR>,
    {
        self.add_many_mapped(values, std::convert::identity)
    }

    /// Adds new values from an iterator to the repeated field after mapping their value.
    pub fn add_many_mapped<'a, I, IT, F, R>(&mut self, values: I, map: F) -> ProtoResult<()>
    where
        I: IntoIterator<Item = IT>,
        IT: 'a,
        F: Fn(IT) -> R,
        R: Borrow<VR>,
    {
        if VP::packable() && self.can_pack {
            self.writer.write_tag(self.packed_field_tag)?;
            self.writer.track_message(|writer| {
                for value in values {
                    let value = map(value);
                    VP::write_value(writer, value.borrow())?;
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
