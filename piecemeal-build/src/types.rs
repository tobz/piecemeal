use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use crate::errors::{Error, Result};
use crate::keywords::sanitize_keyword;
use crate::parser::file_descriptor;

/// Converts a PascalCase string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    #[default]
    Proto2,
    Proto3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    Proto2Frequency(Proto2Frequency),
    Proto3Frequency(Proto3Frequency),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proto2Frequency {
    Optional,
    Repeated,
    Required,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proto3Frequency {
    Optional,
    Repeated,
    Default,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedType {
    SingularType,
    ArrayLikeType,
    Map,
}

impl Frequency {
    pub fn is_map(&self) -> bool {
        matches!(
            self,
            Frequency::Proto2Frequency(Proto2Frequency::Map)
                | Frequency::Proto3Frequency(Proto3Frequency::Map)
        )
    }

    pub fn is_optional(&self) -> bool {
        matches!(
            self,
            Frequency::Proto2Frequency(Proto2Frequency::Optional)
                | Frequency::Proto3Frequency(Proto3Frequency::Optional)
        )
    }

    pub fn is_repeated(&self) -> bool {
        matches!(
            self,
            Frequency::Proto2Frequency(Proto2Frequency::Repeated)
                | Frequency::Proto3Frequency(Proto3Frequency::Repeated)
        )
    }
}

impl From<Frequency> for GeneratedType {
    fn from(value: Frequency) -> Self {
        if value.is_map() {
            GeneratedType::Map
        } else if value.is_repeated() {
            GeneratedType::ArrayLikeType
        } else {
            GeneratedType::SingularType
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct MessageIndex {
    indexes: Vec<usize>,
}

impl fmt::Debug for MessageIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        f.debug_set().entries(self.indexes.iter()).finish()
    }
}

impl MessageIndex {
    pub fn get_message<'a>(&self, desc: &'a FileDescriptor) -> &'a Message {
        let first_message = self.indexes.first().and_then(|i| desc.messages.get(*i));
        self.indexes
            .iter()
            .skip(1)
            .fold(first_message, |cur, next| {
                cur.and_then(|msg| msg.messages.get(*next))
            })
            .expect("Message index not found")
    }

    fn get_message_mut<'a>(&self, desc: &'a mut FileDescriptor) -> &'a mut Message {
        let first_message = self
            .indexes
            .first()
            .and_then(move |i| desc.messages.get_mut(*i));
        self.indexes
            .iter()
            .skip(1)
            .fold(first_message, |cur, next| {
                cur.and_then(|msg| msg.messages.get_mut(*next))
            })
            .expect("Message index not found")
    }

    fn push(&mut self, i: usize) {
        self.indexes.push(i);
    }

    fn pop(&mut self) {
        self.indexes.pop();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EnumIndex {
    msg_index: MessageIndex,
    index: usize,
}

impl EnumIndex {
    pub fn get_enum<'a>(&self, desc: &'a FileDescriptor) -> &'a Enumerator {
        let enums = if self.msg_index.indexes.is_empty() {
            &desc.enums
        } else {
            &self.msg_index.get_message(desc).enums
        };
        enums.get(self.index).expect("Enum index not found")
    }
}

#[derive(Eq, PartialEq)]
pub enum FieldCategory {
    /// A scalar value that can generally be written as-is
    Scalar,

    /// A message value that is potentially written as a series of multiple subfields.
    Message,

    /// A map value that is written as a series of key/value pairs.
    Map,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum(EnumIndex),
    Fixed64,
    Sfixed64,
    Double,
    String,
    Bytes,
    Message(MessageIndex),
    MessageOrEnum(String),
    Fixed32,
    Sfixed32,
    Float,
    Map(Box<FieldType>, Box<FieldType>),
}

impl FieldType {
    fn category(&self) -> FieldCategory {
        match *self {
            FieldType::Message(_) => FieldCategory::Message,
            FieldType::Map(_, _) => FieldCategory::Map,
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
            _ => FieldCategory::Scalar,
        }
    }

    pub fn is_primitive(&self) -> bool {
        !matches!(
            *self,
            FieldType::Message(_) | FieldType::Map(_, _) | FieldType::String | FieldType::Bytes
        )
    }

    fn wire_type_num(&self) -> u32 {
        // TODO: Extract this stuff to a common crate that can be shared between `piecemeal` and
        // `piecemeal-build` so that we're not hard-coding constants and what not in two places.
        match *self {
            FieldType::Int32
            | FieldType::Sint32
            | FieldType::Int64
            | FieldType::Sint64
            | FieldType::Uint32
            | FieldType::Uint64
            | FieldType::Bool
            | FieldType::Enum(_) => 0,
            FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => 1,
            FieldType::String | FieldType::Bytes | FieldType::Message(_) | FieldType::Map(_, _) => {
                2
            }
            FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => 5,
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
        }
    }

    /// Gets the Protocol Buffers type.
    ///
    /// This is distinct from `proto_rust_type`, as it refers to the individual Protocol Buffers types, and not the
    /// condensed helper types (e.g., `Varint`, `Sfixed32`) that we use to encode writing logic into the type system.
    fn proto_type(&self) -> &str {
        match *self {
            FieldType::Int32 => "int32",
            FieldType::Sint32 => "sint32",
            FieldType::Int64 => "int64",
            FieldType::Sint64 => "sint64",
            FieldType::Uint32 => "uint32",
            FieldType::Uint64 => "uint64",
            FieldType::Bool => "bool",
            FieldType::Enum(_) => "enum",
            FieldType::Fixed32 => "fixed32",
            FieldType::Sfixed32 => "sfixed32",
            FieldType::Float => "float",
            FieldType::Fixed64 => "fixed64",
            FieldType::Sfixed64 => "sfixed64",
            FieldType::Double => "double",
            FieldType::String => "string",
            FieldType::Bytes => "bytes",
            FieldType::Message(_) => "message",
            FieldType::Map(_, _) => "map",
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
        }
    }

    /// Gets the Rust-specific Protocol Buffers type.
    ///
    /// This is distinct from `proto_type`, as it refers to the condensed helper types (e.g., `Varint`, `Sfixed32`) that
    /// we use to encode writing logic into the type system, and not the Protocol Buffers types themselves.
    fn proto_rust_type(&self) -> &str {
        match *self {
            FieldType::Bool
            | FieldType::Int32
            | FieldType::Int64
            | FieldType::Uint32
            | FieldType::Uint64
            | FieldType::Enum(_) => "Varint",
            FieldType::Sint32 => "Sint32",
            FieldType::Sint64 => "Sint64",
            FieldType::Fixed32 => "Fixed32",
            FieldType::Fixed64 => "Fixed64",
            FieldType::Sfixed32 | FieldType::Float => "Sfixed32",
            FieldType::Sfixed64 | FieldType::Double => "Sfixed64",
            FieldType::String | FieldType::Bytes => "Bytes",
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
            _ => unreachable!("not a scalar type"),
        }
    }

    /// Gets the Rust type for the given field type as it would exist in a generated message struct.
    ///
    /// Compared to `write_rust_type`, this specifically covers cases where a field needs to be borrowed and can't be
    /// trivially copied, such as strings, bytes, messages, and maps.
    fn struct_rust_type(&self, desc: &FileDescriptor) -> String {
        match *self {
            FieldType::Int32 | FieldType::Sint32 | FieldType::Sfixed32 => "i32".to_string(),
            FieldType::Int64 | FieldType::Sint64 | FieldType::Sfixed64 => "i64".to_string(),
            FieldType::Uint32 | FieldType::Fixed32 => "u32".to_string(),
            FieldType::Uint64 | FieldType::Fixed64 => "u64".to_string(),
            FieldType::Double => "f64".to_string(),
            FieldType::Float => "f32".to_string(),
            FieldType::String => "String".to_string(),
            FieldType::Bytes => "Vec<u8>".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::Enum(ref e) => {
                let e = e.get_enum(desc);
                format!("{}{}", e.get_modules(desc), e.name)
            }
            FieldType::Message(ref msg) => {
                let m = msg.get_message(desc);
                format!("{}{}", m.get_modules(desc), m.name)
            }
            FieldType::Map(ref key, ref value) => format!(
                "KVMap<{}, {}>",
                key.struct_rust_type(desc),
                value.struct_rust_type(desc)
            ),
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
        }
    }

    pub fn message(&self) -> Option<&MessageIndex> {
        if let FieldType::Message(m) = self {
            Some(m)
        } else {
            None
        }
    }

    pub fn map(&self) -> Option<(&FieldType, &FieldType)> {
        if let FieldType::Map(k, v) = self {
            Some((k, v))
        } else {
            None
        }
    }

    /// Gets the Rust type for the given field type as it would be passed in when writing the field.
    ///
    /// This is generally used when writing a field in a message builder, as it relates to the form that callers will
    /// most likely be using, rather than what would be needed to hold the value in a struct.
    ///
    /// Specifically covers scalar types, as other complex types (messages and maps) have dedicated builders.
    fn write_rust_type(&self, desc: &FileDescriptor) -> String {
        match self {
            FieldType::Int32 | FieldType::Sint32 | FieldType::Sfixed32 => "i32".to_string(),
            FieldType::Int64 | FieldType::Sint64 | FieldType::Sfixed64 => "i64".to_string(),
            FieldType::Uint32 | FieldType::Fixed32 => "u32".to_string(),
            FieldType::Uint64 | FieldType::Fixed64 => "u64".to_string(),
            FieldType::Double => "f64".to_string(),
            FieldType::Float => "f32".to_string(),
            FieldType::String => "str".to_string(),
            FieldType::Bytes => "[u8]".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::Enum(ei) => {
                let e = ei.get_enum(desc);
                format!("{}{}", e.get_modules(desc), e.name)
            }
            FieldType::MessageOrEnum(_) => unreachable!("message / enum not resolved"),
            _ => unreachable!("not a scalar type"),
        }
    }

    fn get_size(&self, s: &str) -> String {
        match *self {
            FieldType::Int32
            | FieldType::Int64
            | FieldType::Uint32
            | FieldType::Uint64
            | FieldType::Bool
            | FieldType::Enum(_) => format!("sizeof_varint({} as u64)", s),
            FieldType::Sint32 => format!("sizeof_sint32({})", s),
            FieldType::Sint64 => format!("sizeof_sint64({})", s),

            FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => "8".to_string(),
            FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => "4".to_string(),

            FieldType::String | FieldType::Bytes => format!("sizeof_len({}.len())", s),

            FieldType::Message(_) => format!("sizeof_len({}.get_size())", s),

            FieldType::Map(ref k, ref v) => {
                format!("2 + {} + {}", k.get_size("k"), v.get_size("v"))
            }
            FieldType::MessageOrEnum(_) => unreachable!("Message / Enum not resolved"),
        }
    }

    fn get_write(&self, s: &str, needs_deref: bool) -> String {
        let with_deref = if needs_deref { "*" } else { "" };
        match *self {
            FieldType::Enum(_) => format!("write_enum({}{} as i32)", with_deref, s),

            FieldType::Int32
            | FieldType::Sint32
            | FieldType::Int64
            | FieldType::Sint64
            | FieldType::Uint32
            | FieldType::Uint64
            | FieldType::Bool
            | FieldType::Fixed64
            | FieldType::Sfixed64
            | FieldType::Double
            | FieldType::Fixed32
            | FieldType::Sfixed32
            | FieldType::Float => format!("write_{}({}{})", self.proto_type(), with_deref, s),

            FieldType::String => format!("write_string({})", s),
            FieldType::Bytes => format!("write_bytes({})", s),

            FieldType::Message(_) if needs_deref => format!("write_message(&*({}))", s),
            FieldType::Message(_) => format!("write_message({})", s),

            FieldType::Map(ref k, ref v) => format!(
                "write_map({}, {}, |w| w.{}, {}, |w| w.{})",
                self.get_size(""),
                tag(1, k),
                k.get_write("k", false),
                tag(2, v),
                v.get_write("v", false)
            ),
            FieldType::MessageOrEnum(_) => unreachable!("Message / Enum not resolved"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub frequency: Frequency,
    pub typ: FieldType,
    pub number: i32,
    pub default: Option<String>,
    pub packed: Option<bool>,
    pub boxed: bool,
    pub deprecated: bool,
}

impl Field {
    fn packed(&self) -> bool {
        self.packed.unwrap_or(false)
    }

    fn tag(&self) -> u32 {
        tag(self.number as u32, &self.typ)
    }
}

fn get_modules(module: &str, imported: bool, desc: &FileDescriptor) -> String {
    let skip = usize::from(desc.package.is_empty() && !imported);
    module
        .split('.')
        .filter(|p| !p.is_empty())
        .skip(skip)
        .fold(String::new(), |mut s, p| {
            s.push_str(p);
            s.push_str("::");
            s
        })
}

#[derive(Debug, Clone, Default)]
pub struct Extend {
    /// The message being extended.
    pub name: String,
    /// All fields that are being added to the extended message.
    pub fields: Vec<Field>,
}

impl Extend {}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub oneofs: Vec<OneOf>,
    pub reserved_nums: Option<Vec<i32>>,
    pub reserved_names: Option<Vec<String>>,
    pub imported: bool,
    pub package: String,        // package from imports + nested items
    pub messages: Vec<Message>, // nested messages
    pub enums: Vec<Enumerator>, // nested enums
    pub module: String,         // 'package' corresponding to actual generated Rust module
    pub path: PathBuf,
    pub import: PathBuf,
    pub index: MessageIndex,
    /// Allowed extensions for this message, None if no extensions.
    pub extensions: Option<Extensions>,
}

impl Message {
    fn convert_field_types(&mut self, from: &FieldType, to: &FieldType) {
        for f in self.all_fields_mut().filter(|f| f.typ == *from) {
            f.typ = to.clone();
        }

        // If that type is a map with the fieldtype, it must also be converted.
        for f in self.all_fields_mut() {
            let new_type: FieldType = match f.typ {
                FieldType::Map(ref mut key, ref mut value)
                    if **key == *from && **value == *from =>
                {
                    FieldType::Map(Box::new(to.clone()), Box::new(to.clone()))
                }
                FieldType::Map(ref mut key, ref mut value) if **key == *from => {
                    FieldType::Map(Box::new(to.clone()), value.clone())
                }
                FieldType::Map(ref mut key, ref mut value) if **value == *from => {
                    FieldType::Map(key.clone(), Box::new(to.clone()))
                }
                ref other => other.clone(),
            };
            f.typ = new_type;
        }

        for message in &mut self.messages {
            message.convert_field_types(from, to);
        }
    }

    fn set_imported(&mut self) {
        self.imported = true;
        for m in self.messages.iter_mut() {
            m.set_imported();
        }
        for e in self.enums.iter_mut() {
            e.imported = true;
        }
    }

    fn get_modules(&self, desc: &FileDescriptor) -> String {
        get_modules(&self.module, self.imported, desc)
    }

    fn write_common_uses<W: Write>(w: &mut W, messages: &[Message]) -> Result<()> {
        // Check if any map has scalar values (uses GenericMapBuilder)
        let has_scalar_value_maps = messages.iter().filter(|m| !m.imported).any(|m| {
            m.all_fields().any(|f| {
                if let Some((_, v)) = f.typ.map() {
                    v.category() == FieldCategory::Scalar
                } else {
                    false
                }
            })
        });

        // Check if any map has message values (uses MessageMapBuilder)
        let has_message_value_maps = messages.iter().filter(|m| !m.imported).any(|m| {
            m.all_fields().any(|f| {
                if let Some((_, v)) = f.typ.map() {
                    v.category() == FieldCategory::Message
                } else {
                    false
                }
            })
        });

        if has_scalar_value_maps {
            writeln!(w, "use ::piecemeal::GenericMapBuilder;")?;
        }

        if has_message_value_maps {
            writeln!(w, "use ::piecemeal::MessageMapBuilder;")?;
        }

        if messages
            .iter()
            .filter(|m| !m.imported)
            .any(|m| m.all_fields().any(|f| f.frequency.is_repeated()))
        {
            writeln!(w, "use ::piecemeal::RepeatedBuilder;")?;
        }

        Ok(())
    }

    fn write<W: Write>(&self, w: &mut W, desc: &FileDescriptor) -> Result<()> {
        println!("Writing message {}{}", self.get_modules(desc), self.name);
        writeln!(w)?;

        self.write_message_builder(w, desc)?;

        if !(self.messages.is_empty() && self.enums.is_empty()) {
            writeln!(w)?;
            writeln!(w, "pub mod {} {{", to_snake_case(&self.name))?;
            writeln!(w)?;

            Self::write_common_uses(w, &self.messages)?;

            if !self.messages.is_empty() {
                writeln!(w, "use super::*;")?;
            }
            for m in &self.messages {
                m.write(w, desc)?;
            }
            for e in &self.enums {
                e.write(w)?;
            }

            writeln!(w)?;
            writeln!(w, "}}")?;
        }

        Ok(())
    }

    fn write_message_builder<W: Write>(&self, w: &mut W, desc: &FileDescriptor) -> Result<()> {
        writeln!(w, "pub struct {};", self.name)?;
        writeln!(w)?;
        writeln!(
            w,
            "pub struct {}Builder<'w, S: ScratchBuffer> {{",
            self.name
        )?;
        writeln!(w, "    writer: &'w mut ScratchWriter<S>")?;
        writeln!(w, "}}")?;
        writeln!(w)?;
        writeln!(
            w,
            "impl<'w, S: ScratchBuffer> {}Builder<'w, S> {{",
            self.name
        )?;
        writeln!(
            w,
            "    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {{"
        )?;
        writeln!(w, "        Self {{ writer }}")?;
        writeln!(w, "    }}")?;
        for field in &self.fields {
            writeln!(w)?;
            self.write_message_builder_field(w, field, desc)?;
        }
        writeln!(w)?;
        writeln!(
            w,
            "    pub fn finish<W: Writer>(self, output: &mut W) -> ProtoResult<()> {{"
        )?;
        writeln!(w, "        self.writer.finish(output, false)")?;
        writeln!(w, "    }}")?;
        writeln!(w)?;
        writeln!(
            w,
            "    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> ProtoResult<()> {{"
        )?;
        writeln!(w, "        self.writer.finish(output, true)")?;
        writeln!(w, "    }}")?;
        writeln!(w, "}}")?;

        writeln!(w)?;
        writeln!(
            w,
            "impl<S: ScratchBuffer> MessageBuilderBase<S> for {} {{",
            self.name
        )?;
        writeln!(
            w,
            "    type Builder<'a> = {}Builder<'a, S> where S: 'a;",
            self.name
        )?;
        writeln!(w, "}}")?;
        writeln!(w)?;
        writeln!(
            w,
            "impl<S: ScratchBuffer> MessageBuilder<S> for {} {{",
            self.name
        )?;
        writeln!(
            w,
            "    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {{"
        )?;
        writeln!(w, "        {}Builder::new(writer)", self.name)?;
        writeln!(w, "    }}")?;
        writeln!(w, "}}")?;

        Ok(())
    }

    fn write_message_builder_field<W: Write>(
        &self,
        w: &mut W,
        field: &Field,
        desc: &FileDescriptor,
    ) -> Result<()> {
        match field.typ.category() {
            FieldCategory::Scalar => self.write_message_builder_field_scalar(w, field, desc),
            FieldCategory::Message => self.write_message_builder_field_message(w, field, desc),
            FieldCategory::Map => self.write_message_builder_field_map(w, field, desc),
        }
    }

    fn write_message_builder_field_scalar<W: Write>(
        &self,
        w: &mut W,
        field: &Field,
        desc: &FileDescriptor,
    ) -> Result<()> {
        let is_repeated = field.frequency.is_repeated();
        let proto_typ = field.typ.proto_rust_type();
        let value_typ = field.typ.write_rust_type(desc);

        if is_repeated {
            writeln!(
                w,
                "    pub fn {}<F>(&mut self, f: F) -> ProtoResult<&mut Self>",
                field.name
            )?;
            writeln!(w, "    where")?;
            writeln!(
                w,
                "        F: for<'a> FnOnce(&mut RepeatedBuilder<'a, S, {}, {}>) -> ProtoResult<()>,",
                proto_typ, value_typ
            )?;
            writeln!(w, "    {{")?;
            writeln!(
                w,
                "        let mut repeated_builder = RepeatedBuilder::new({}, {},self.writer);",
                field.number,
                field.packed()
            )?;
            writeln!(w, "        f(&mut repeated_builder)?;")?;
            writeln!(w, "        Ok(self)")?;
            writeln!(w, "    }}")?;
        } else {
            // Field isn't repeated, so just a basic write.
            let value_typ = if !field.typ.is_primitive() {
                format!("&{}", value_typ)
            } else {
                value_typ.to_string()
            };
            writeln!(
                w,
                "    pub fn {}(&mut self, value: {}) -> ProtoResult<&mut Self> {{",
                field.name, value_typ
            )?;
            writeln!(
                w,
                "        self.writer.write_with_tag({}, |w| w.{})?;",
                field.tag(),
                field.typ.get_write("value", false)
            )?;
            writeln!(w, "        Ok(self)")?;
            writeln!(w, "    }}")?;
        }
        Ok(())
    }

    fn write_message_builder_field_message<W: Write>(
        &self,
        w: &mut W,
        field: &Field,
        desc: &FileDescriptor,
    ) -> Result<()> {
        let typ = field.typ.struct_rust_type(desc);

        let method_name = match field.frequency.is_repeated() {
            true => format!("add_{}", field.name),
            false => field.name.clone(),
        };

        writeln!(
            w,
            "    pub fn {}<F>(&mut self, f: F) -> ProtoResult<&mut Self>",
            method_name
        )?;
        writeln!(w, "    where")?;
        writeln!(
            w,
            "        F: for<'a> FnOnce(&mut {}Builder<'a, S>) -> ProtoResult<()>",
            typ
        )?;
        writeln!(w, "    {{")?;
        writeln!(w, "        {{")?;
        writeln!(w, "            self.writer.write_tag({})?;", field.tag())?;
        writeln!(w, "            self.writer.track_message(|sw| {{")?;
        writeln!(
            w,
            "              let mut msg_builder = {}Builder::new(sw);",
            typ
        )?;
        writeln!(w, "              f(&mut msg_builder)")?;
        writeln!(w, "            }})?;")?;
        writeln!(w, "        }}")?;
        writeln!(w, "        Ok(self)")?;
        writeln!(w, "    }}")?;
        Ok(())
    }

    fn write_message_builder_field_map<W: Write>(
        &self,
        w: &mut W,
        field: &Field,
        desc: &FileDescriptor,
    ) -> Result<()> {
        let (key_field_type, value_field_type) = field.typ.map().expect("field should be a map");

        // Map keys must always be scalar types per the protobuf spec
        if key_field_type.category() != FieldCategory::Scalar {
            return Err(Error::InvalidMessage(
                "map keys must be scalar types".to_string(),
            ));
        }

        let key_typ = key_field_type.write_rust_type(desc);

        match value_field_type.category() {
            FieldCategory::Scalar => {
                // Scalar-to-scalar map: use GenericMapBuilder
                let value_typ = value_field_type.write_rust_type(desc);
                writeln!(
                    w,
                    "    pub fn {}(&mut self) -> GenericMapBuilder<'_, S, {}, {}> {{",
                    field.name, key_typ, value_typ
                )?;
                writeln!(
                    w,
                    "        GenericMapBuilder::new({}, self.writer)",
                    field.number
                )?;
                writeln!(w, "    }}")?;
            }
            FieldCategory::Message => {
                // Scalar-to-message map: return MessageMapBuilder with builder type
                let value_typ = value_field_type.struct_rust_type(desc);
                writeln!(
                    w,
                    "    pub fn {}(&mut self) -> MessageMapBuilder<'_, S, {}, {}> {{",
                    field.name, key_typ, value_typ
                )?;
                writeln!(
                    w,
                    "        MessageMapBuilder::new({}, self.writer)",
                    field.number
                )?;
                writeln!(w, "    }}")?;
            }
            FieldCategory::Map => {
                return Err(Error::InvalidMessage(
                    "map values cannot be maps".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn sanity_checks(&self, desc: &FileDescriptor) -> Result<()> {
        // We don't yet support oneof fields.
        if !self.oneofs.is_empty() {
            return Err(Error::InvalidMessage(
                "oneof fields are not yet supported".to_string(),
            ));
        }

        for f in self.all_fields() {
            // check reserved
            if self
                .reserved_names
                .as_ref()
                .is_some_and(|names| names.contains(&f.name))
                || self
                    .reserved_nums
                    .as_ref()
                    .is_some_and(|nums| nums.contains(&f.number))
            {
                return Err(Error::InvalidMessage(format!(
                    "Error in message {}\n\
                     Field {:?} conflict with reserved fields",
                    self.name, f
                )));
            }

            // check default enums
            if let Some(var) = f.default.as_ref()
                && let FieldType::Enum(ref e) = f.typ
            {
                let e = e.get_enum(desc);
                e.fields.iter().find(|(name, _)| name == var).ok_or_else(|| {
                        Error::InvalidDefaultEnum(format!(
                            "Error in message {}\n\
                                Enum field {:?} has a default value '{}' which is not valid for enum index {:?}",
                            self.name, f, var, e
                        ))
                    })?;
            }
        }
        Ok(())
    }

    fn set_package(&mut self, package: &str, module: &str) {
        // The complication here is that the _package_ (as declared in the proto file) does
        // not directly map to the _module_. For example, the package 'a.A' where A is a
        // message will be the module 'a.a', since we can't reuse the message name A as
        // the submodule containing nested items. Also, protos with empty packages always
        // have a module corresponding to the file name.
        let (child_package, child_module) = if package.is_empty() {
            self.module = module.to_string();
            (
                self.name.clone(),
                format!("{}.{}", module, to_snake_case(&self.name)),
            )
        } else {
            self.package = package.to_string();
            self.module = module.to_string();
            (
                format!("{}.{}", package, self.name),
                format!("{}.{}", module, to_snake_case(&self.name)),
            )
        };

        for m in &mut self.messages {
            m.set_package(&child_package, &child_module);
        }
        for m in &mut self.enums {
            m.set_package(&child_package, &child_module);
        }
    }

    fn set_repeated_as_packed(&mut self) {
        for f in self.all_fields_mut() {
            if f.packed.is_none() && f.frequency.is_repeated() && f.typ.is_primitive() {
                f.packed = Some(true);
            }
        }
    }

    fn sanitize_names(&mut self) {
        sanitize_keyword(&mut self.name);
        sanitize_keyword(&mut self.package);
        for f in self.fields.iter_mut() {
            sanitize_keyword(&mut f.name);
        }
        for m in &mut self.messages {
            m.sanitize_names();
        }
        for e in &mut self.enums {
            e.sanitize_names();
        }
    }

    /// Return an iterator producing references to all the `Field`s of `self`,
    /// including both direct and `oneof` fields.
    pub fn all_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    /// Return an iterator producing mutable references to all the `Field`s of
    /// `self`, including both direct and `oneof` fields.
    fn all_fields_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.fields.iter_mut()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Extensions {
    pub from: i32,
    /// Max number is 536,870,911 (2^29 - 1), as defined in the
    /// protobuf docs
    pub to: i32,
}

impl Extensions {
    /// The max field number that can be used as an extension.
    pub fn max() -> i32 {
        536870911
    }
}

#[derive(Debug, Clone, Default)]
pub struct Enumerator {
    pub name: String,
    pub fields: Vec<(String, i32)>,
    pub fully_qualified_fields: Vec<(String, i32)>,
    pub partially_qualified_fields: Vec<(String, i32)>,
    pub imported: bool,
    pub package: String,
    pub module: String,
    pub path: PathBuf,
    pub import: PathBuf,
    pub index: EnumIndex,
}

impl Enumerator {
    fn set_package(&mut self, package: &str, module: &str) {
        self.package = package.to_string();
        self.module = module.to_string();
        self.partially_qualified_fields = self
            .fields
            .iter()
            .map(|f| (format!("{}::{}", &self.name, f.0), f.1))
            .collect();
        self.fully_qualified_fields = self
            .partially_qualified_fields
            .iter()
            .map(|pqf| {
                let fqf = if self.module.is_empty() {
                    pqf.0.clone()
                } else {
                    format!("{}::{}", self.module.replace('.', "::"), pqf.0)
                };
                (fqf, pqf.1)
            })
            .collect();
    }

    fn sanitize_names(&mut self) {
        sanitize_keyword(&mut self.name);
        sanitize_keyword(&mut self.package);
        for f in self.fields.iter_mut() {
            sanitize_keyword(&mut f.0);
        }
    }

    fn get_modules(&self, desc: &FileDescriptor) -> String {
        get_modules(&self.module, self.imported, desc)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        println!("Writing enum {}", self.name);
        writeln!(w)?;
        self.write_definition(w)?;
        writeln!(w)?;
        if self.fields.is_empty() {
            Ok(())
        } else {
            self.write_impl_default(w)?;
            writeln!(w)?;
            self.write_from_i32(w)?;
            writeln!(w)?;
            self.write_from_str(w)
        }
    }

    fn write_definition<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w, "#[derive(Debug, PartialEq, Eq, Clone, Copy)]")?;
        writeln!(w, "pub enum {} {{", self.name)?;
        for (f, number) in &self.fields {
            writeln!(w, "    {} = {},", f, number)?;
        }
        writeln!(w, "}}")?;
        Ok(())
    }

    fn write_impl_default<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w, "impl Default for {} {{", self.name)?;
        writeln!(w, "    fn default() -> Self {{")?;
        // TODO: check with default field and return error if there is no field
        writeln!(w, "        {}", self.partially_qualified_fields[0].0)?;
        writeln!(w, "    }}")?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn write_from_i32<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w, "impl From<i32> for {} {{", self.name)?;
        writeln!(w, "    fn from(i: i32) -> Self {{")?;
        writeln!(w, "        match i {{")?;
        for (f, number) in &self.fields {
            writeln!(w, "            {} => {}::{},", number, self.name, f)?;
        }
        writeln!(w, "            _ => Self::default(),")?;
        writeln!(w, "        }}")?;
        writeln!(w, "    }}")?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn write_from_str<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w, "impl<'a> From<&'a str> for {} {{", self.name)?;
        writeln!(w, "    fn from(s: &'a str) -> Self {{")?;
        writeln!(w, "        match s {{")?;
        for (f, _) in &self.fields {
            writeln!(w, "            {:?} => {}::{},", f, self.name, f)?;
        }
        writeln!(w, "            _ => Self::default(),")?;
        writeln!(w, "        }}")?;
        writeln!(w, "    }}")?;
        writeln!(w, "}}")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct OneOf {
    pub name: String,
    pub fields: Vec<Field>,
    pub package: String,
    pub module: String,
    pub imported: bool,
}

pub struct Config {
    pub in_file: PathBuf,
    pub out_dir: PathBuf,
    pub single_module: bool,
    pub import_search_path: Vec<PathBuf>,
    pub error_cycle: bool,
    pub headers: bool,
    pub add_deprecated_fields: bool,
}

#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    pub import_paths: Vec<PathBuf>,
    pub package: String,
    pub syntax: Syntax,
    pub messages: Vec<Message>,
    pub message_extends: Vec<Extend>,
    pub enums: Vec<Enumerator>,
    pub module: String,
}

impl FileDescriptor {
    pub fn run(configs: &[Config]) -> Result<()> {
        for config in configs {
            Self::write_proto(config)?
        }
        Ok(())
    }

    pub fn write_proto(config: &Config) -> Result<()> {
        let mut desc = FileDescriptor::read_proto(&config.in_file, &config.import_search_path)?;

        if desc.messages.is_empty() && desc.enums.is_empty() {
            // There could had been unsupported structures, so bail early
            return Err(Error::EmptyRead);
        }

        desc.resolve_types()?;
        desc.break_cycles(config.error_cycle)?;
        desc.sanity_checks()?;
        desc.set_defaults()?;
        desc.sanitize_names();

        if config.single_module {
            desc.package = "".to_string();
        }

        let (prefix, file_package) = split_package(&desc.package);

        let mut file_stem = if file_package.is_empty() {
            get_file_stem(&config.in_file)?
        } else {
            file_package.to_string()
        };

        if !file_package.is_empty() {
            sanitize_keyword(&mut file_stem);
        }
        let mut out_file = config.out_dir.join(format!("{}.rs", file_stem));

        if !prefix.is_empty() {
            use std::fs::create_dir_all;
            // e.g. package is a.b; we need to create directory 'a' and insert it into the path
            let file = PathBuf::from(out_file.file_name().unwrap());
            out_file.pop();
            for p in prefix.split('.') {
                out_file.push(p);

                if !out_file.exists() {
                    create_dir_all(&out_file)?;
                    update_mod_file(&out_file)?;
                }
            }
            out_file.push(file);
        }

        let name = config.in_file.file_name().and_then(|e| e.to_str()).unwrap();

        let mut w = BufWriter::new(File::create(&out_file)?);
        desc.write(&mut w, name, config)?;
        update_mod_file(&out_file)
    }

    pub fn convert_field_types(&mut self, from: &FieldType, to: &FieldType) {
        // Messages and enums are the only structures with types
        for m in &mut self.messages {
            m.convert_field_types(from, to);
        }
    }

    /// Opens a proto file, reads it and returns raw parsed data
    pub fn read_proto(in_file: &Path, import_search_path: &[PathBuf]) -> Result<FileDescriptor> {
        let file = std::fs::read_to_string(in_file)?;
        let (rem, mut desc) = file_descriptor(&file).map_err(Error::Nom)?;
        let rem = rem.trim();
        if !rem.is_empty() {
            return Err(Error::TrailingGarbage(rem.chars().take(50).collect()));
        }
        for m in &mut desc.messages {
            if m.path.as_os_str().is_empty() {
                m.path = in_file.to_path_buf();
                if !import_search_path.is_empty()
                    && let Ok(p) = m.path.clone().strip_prefix(&import_search_path[0])
                {
                    m.import = p.to_path_buf();
                }
            }
        }
        // proto files with no packages are given an implicit module,
        // since every generated Rust source file represents a module
        desc.module = if desc.package.is_empty() {
            get_file_stem(in_file)?
        } else {
            desc.package.clone()
        };

        desc.fetch_imports(in_file, import_search_path)?;
        Ok(desc)
    }

    fn sanity_checks(&self) -> Result<()> {
        for m in &self.messages {
            m.sanity_checks(self)?;
        }
        Ok(())
    }

    /// Get messages and enums from imports
    fn fetch_imports(&mut self, in_file: &Path, import_search_path: &[PathBuf]) -> Result<()> {
        for m in &mut self.messages {
            m.set_package(&self.package, &self.module);
        }
        for m in &mut self.enums {
            m.set_package(&self.package, &self.module);
        }

        for import in &self.import_paths {
            // this is the same logic as the C preprocessor;
            // if the include path item is absolute, then append the filename,
            // otherwise it is always relative to the file.
            let mut matching_file = None;
            for path in import_search_path {
                let candidate = if path.is_absolute() {
                    path.join(import)
                } else {
                    in_file
                        .parent()
                        .map_or_else(|| path.join(import), |p| p.join(path).join(import))
                };
                if candidate.exists() {
                    matching_file = Some(candidate);
                    break;
                }
            }
            if matching_file.is_none() {
                return Err(Error::InvalidImport(format!(
                    "file {} not found on import path",
                    import.display()
                )));
            }
            let proto_file = matching_file.unwrap();
            let mut f = FileDescriptor::read_proto(&proto_file, import_search_path)?;

            // if the proto has a packge then the names will be prefixed
            let package = f.package.clone();
            let module = f.module.clone();
            self.messages.extend(f.messages.drain(..).map(|mut m| {
                if m.package.is_empty() {
                    m.set_package(&package, &module);
                }
                if m.path.as_os_str().is_empty() {
                    m.path.clone_from(&proto_file);
                }
                if m.import.as_os_str().is_empty() {
                    m.import.clone_from(import);
                }
                m.set_imported();
                m
            }));
            self.enums.extend(f.enums.drain(..).map(|mut e| {
                if e.package.is_empty() {
                    e.set_package(&package, &module);
                }
                if e.path.as_os_str().is_empty() {
                    e.path.clone_from(&proto_file);
                }
                if e.import.as_os_str().is_empty() {
                    e.import.clone_from(import);
                }
                e.imported = true;
                e
            }));
        }
        Ok(())
    }

    fn set_defaults(&mut self) -> Result<()> {
        // Set specific default behavior for messages/fields if we're using Protocol Buffers v3 syntax.
        if let Syntax::Proto3 = self.syntax {
            let mut nested_messages = VecDeque::new();

            // Go through the top-level first, collecting the first line of any nested messages
            // within those. We'll go through the nested messages afterwards to fully crawl the file
            // descriptor and ensure all messages have been visited.
            for m in &mut self.messages {
                m.set_repeated_as_packed();

                for f in m.all_fields() {
                    if let Some(m_idx) = f.typ.message() {
                        nested_messages.push_back(m_idx.clone());
                    }
                }
            }

            while let Some(m) = nested_messages.pop_front() {
                let m = m.get_message_mut(self);
                m.set_repeated_as_packed();

                for f in m.all_fields() {
                    if let Some(m_idx) = f.typ.message() {
                        nested_messages.push_back(m_idx.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn sanitize_names(&mut self) {
        for m in &mut self.messages {
            m.sanitize_names();
        }
        for e in &mut self.enums {
            e.sanitize_names();
        }
    }

    /// Breaks cycles by adding boxes when necessary
    fn break_cycles(&mut self, error_cycle: bool) -> Result<()> {
        // get strongly connected components
        let sccs = self.sccs();

        fn is_cycle(scc: &[MessageIndex], desc: &FileDescriptor) -> bool {
            scc.iter()
                .map(|m| m.get_message(desc))
                .flat_map(|m| m.all_fields())
                .filter(|f| !f.boxed)
                .filter_map(|f| f.typ.message())
                .any(|m| scc.contains(m))
        }

        // sccs are sub DFS trees so if there is a edge connecting a node to
        // another node higher in the scc list, then this is a cycle. (Note that
        // we may have several cycles per scc).
        //
        // Technically we only need to box one edge (optional field) per cycle to
        // have Sized structs. Unfortunately, scc root depend on the order we
        // traverse the graph so such a field is not guaranteed to always be the same.
        //
        // For now, we decide (see discussion in #121) to box all optional fields
        // within a scc. We favor generated code stability over performance.
        for scc in &sccs {
            debug!("scc: {:?}", scc);
            for (i, v) in scc.iter().enumerate() {
                // cycles with v as root
                let cycles = v
                    .get_message(self)
                    .all_fields()
                    .filter_map(|f| f.typ.message())
                    .filter_map(|m| scc[i..].iter().position(|n| n == m))
                    .collect::<Vec<_>>();
                for cycle in cycles {
                    let cycle = &scc[i..i + cycle + 1];
                    debug!("cycle: {:?}", &cycle);
                    for v in cycle {
                        for f in v
                            .get_message_mut(self)
                            .all_fields_mut()
                            .filter(|f| f.frequency.is_optional())
                            .filter(|f| f.typ.message().is_some_and(|m| cycle.contains(m)))
                        {
                            f.boxed = true;
                        }
                    }
                    if is_cycle(cycle, self) {
                        if error_cycle {
                            return Err(Error::Cycle(
                                cycle
                                    .iter()
                                    .map(|m| m.get_message(self).name.clone())
                                    .collect(),
                            ));
                        } else {
                            for v in cycle {
                                warn!(
                                    "Unsound proto file would result in infinite size Messages.\n\
                                     Cycle detected in messages {:?}.\n\
                                     Modifying required fields into optional fields",
                                    cycle
                                        .iter()
                                        .map(|m| &m.get_message(self).name)
                                        .collect::<Vec<_>>()
                                );
                                for f in v
                                    .get_message_mut(self)
                                    .all_fields_mut()
                                    .filter(|f| {
                                        !(f.frequency.is_optional() || f.frequency.is_repeated())
                                    })
                                    .filter(|f| f.typ.message().is_some_and(|m| cycle.contains(m)))
                                {
                                    f.boxed = true;
                                    f.frequency = match f.frequency {
                                        Frequency::Proto2Frequency(_) => {
                                            Frequency::Proto2Frequency(Proto2Frequency::Optional)
                                        }
                                        Frequency::Proto3Frequency(_) => {
                                            Frequency::Proto3Frequency(Proto3Frequency::Optional)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_full_names(&mut self) -> (HashMap<String, MessageIndex>, HashMap<String, EnumIndex>) {
        fn rec_full_names(
            m: &mut Message,
            index: &mut MessageIndex,
            full_msgs: &mut HashMap<String, MessageIndex>,
            full_enums: &mut HashMap<String, EnumIndex>,
        ) {
            m.index = index.clone();
            if m.package.is_empty() {
                full_msgs
                    .entry(m.name.clone())
                    .or_insert_with(|| index.clone());
            } else {
                full_msgs
                    .entry(format!("{}.{}", m.package, m.name))
                    .or_insert_with(|| index.clone());
            }
            for (i, e) in m.enums.iter_mut().enumerate() {
                let index = EnumIndex {
                    msg_index: index.clone(),
                    index: i,
                };
                e.index = index.clone();
                full_enums
                    .entry(format!("{}.{}", e.package, e.name))
                    .or_insert(index);
            }
            for (i, m) in m.messages.iter_mut().enumerate() {
                index.push(i);
                rec_full_names(m, index, full_msgs, full_enums);
                index.pop();
            }
        }

        let mut full_msgs = HashMap::new();
        let mut full_enums = HashMap::new();
        let mut index = MessageIndex { indexes: vec![] };
        for (i, m) in self.messages.iter_mut().enumerate() {
            index.push(i);
            rec_full_names(m, &mut index, &mut full_msgs, &mut full_enums);
            index.pop();
        }
        for (i, e) in self.enums.iter_mut().enumerate() {
            let index = EnumIndex {
                msg_index: index.clone(),
                index: i,
            };
            e.index = index.clone();
            if e.package.is_empty() {
                full_enums
                    .entry(e.name.clone())
                    .or_insert_with(|| index.clone());
            } else {
                full_enums
                    .entry(format!("{}.{}", e.package, e.name))
                    .or_insert_with(|| index.clone());
            }
        }
        (full_msgs, full_enums)
    }

    fn resolve_types(&mut self) -> Result<()> {
        let (full_msgs, full_enums) = self.get_full_names();

        fn rec_resolve_types(
            m: &mut Message,
            full_msgs: &HashMap<String, MessageIndex>,
            full_enums: &HashMap<String, EnumIndex>,
        ) -> Result<()> {
            // Interestingly, we can't call all_fields_mut to iterate over the
            // fields here: writing out the field traversal as below lets Rust
            // split m's mutable borrow, permitting the loop body to use fields
            // of `m` other than `fields` and `oneofs`.
            'types: for typ in m
                .fields
                .iter_mut()
                .map(|f| &mut f.typ)
                .flat_map(|typ| match *typ {
                    FieldType::Map(ref mut key, ref mut value) => {
                        vec![&mut **key, &mut **value].into_iter()
                    }
                    _ => vec![typ].into_iter(),
                })
            {
                if let FieldType::MessageOrEnum(name) = typ.clone() {
                    let test_names: Vec<String> = if name.starts_with('.') {
                        vec![name.clone().split_off(1)]
                    } else if m.package.is_empty() {
                        vec![format!("{}.{}", m.name, name), name.clone()]
                    } else {
                        let mut v = vec![
                            format!("{}.{}.{}", m.package, m.name, name),
                            format!("{}.{}", m.package, name),
                        ];
                        for (index, _) in m.package.match_indices('.').rev() {
                            v.push(format!("{}.{}", &m.package[..index], name));
                        }
                        v.push(name.clone());
                        v
                    };
                    for name in &test_names {
                        if let Some(msg) = full_msgs.get(name) {
                            *typ = FieldType::Message(msg.clone());
                            continue 'types;
                        } else if let Some(e) = full_enums.get(name) {
                            *typ = FieldType::Enum(e.clone());
                            continue 'types;
                        }
                    }
                    return Err(Error::MessageOrEnumNotFound(name));
                }
            }
            for m in m.messages.iter_mut() {
                rec_resolve_types(m, full_msgs, full_enums)?;
            }
            Ok(())
        }

        for m in self.messages.iter_mut() {
            rec_resolve_types(m, &full_msgs, &full_enums)?;
        }
        Ok(())
    }

    fn write<W: Write>(&self, w: &mut W, filename: &str, config: &Config) -> Result<()> {
        println!(
            "Found {} messages, and {} enums",
            self.messages.len(),
            self.enums.len()
        );
        if config.headers {
            self.write_headers(w, filename, config)?;
        }
        self.write_package_start(w)?;
        self.write_uses(w)?;
        self.write_imports(w)?;
        self.write_enums(w)?;
        self.write_messages(w)?;
        self.write_package_end(w)?;
        Ok(())
    }

    fn write_headers<W: Write>(&self, w: &mut W, filename: &str, config: &Config) -> Result<()> {
        writeln!(
            w,
            "// Automatically generated Rust module for '{}' file. Do not modify directly.",
            filename
        )?;
        writeln!(w)?;
        writeln!(w, "#![allow(dead_code)]")?;
        writeln!(w, "#![allow(non_snake_case)]")?;
        writeln!(w, "#![allow(non_upper_case_globals)]")?;
        writeln!(w, "#![allow(non_camel_case_types)]")?;
        writeln!(w, "#![allow(unused_imports)]")?;
        writeln!(w, "#![allow(unknown_lints)]")?;
        writeln!(w, "#![allow(clippy::all)]")?;

        if config.add_deprecated_fields {
            writeln!(w, "#![allow(deprecated)]")?;
        }

        writeln!(w, "#![cfg_attr(rustfmt, rustfmt_skip)]")?;
        writeln!(w)?;
        Ok(())
    }

    fn write_package_start<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w)?;
        Ok(())
    }

    fn write_uses<W: Write>(&self, w: &mut W) -> Result<()> {
        Message::write_common_uses(w, &self.messages)?;

        writeln!(
            w,
            "use ::piecemeal::{{helpers::*, types::{{protobuf::*, MessageBuilderBase, MessageBuilder, WireType}}, MapScalar, ScratchBuffer, ScratchWriter, Writer, ProtoResult}};"
        )?;
        Ok(())
    }

    fn write_imports<W: Write>(&self, w: &mut W) -> Result<()> {
        // even if we don't have an explicit package, there is an implicit Rust module
        // This `use` allows us to refer to the package root.
        // NOTE! I'm suppressing not-needed 'use super::*' errors currently!
        let mut depth = self.package.split('.').count();
        if depth == 0 {
            depth = 1;
        }
        write!(w, "use ")?;
        for _ in 0..depth {
            write!(w, "super::")?;
        }
        writeln!(w, "*;")?;
        Ok(())
    }

    fn write_package_end<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w)?;
        Ok(())
    }

    fn write_enums<W: Write>(&self, w: &mut W) -> Result<()> {
        for m in self.enums.iter().filter(|e| !e.imported) {
            println!("Writing enum {}", m.name);
            writeln!(w)?;
            m.write_definition(w)?;
            writeln!(w)?;
            m.write_impl_default(w)?;
            writeln!(w)?;
            m.write_from_i32(w)?;
            writeln!(w)?;
            m.write_from_str(w)?;
        }
        Ok(())
    }

    fn write_messages<W: Write>(&self, w: &mut W) -> Result<()> {
        for m in self.messages.iter().filter(|m| !m.imported) {
            m.write(w, self)?;
        }
        Ok(())
    }
}

/// Calculates the tag value
fn tag(number: u32, typ: &FieldType) -> u32 {
    number << 3 | typ.wire_type_num()
}

/// "" is ("",""), "a" is ("","a"), "a.b" is ("a"."b"), and so forth.
fn split_package(package: &str) -> (&str, &str) {
    if package.is_empty() {
        ("", "")
    } else if let Some(i) = package.rfind('.') {
        (&package[0..i], &package[i + 1..])
    } else {
        ("", package)
    }
}

const MAGIC_HEADER: &str = "// Automatically generated by piecemeal.";

/// Given a file path, create or update the mod.rs file within its folder
fn update_mod_file(path: &Path) -> Result<()> {
    let mut file = path.to_path_buf();
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let name = file.file_stem().unwrap().to_string_lossy().to_string();
    file.pop();
    file.push("mod.rs");
    let matches = "pub mod ";
    let mut present = false;
    let mut exists = false;
    if let Ok(f) = File::open(&file) {
        exists = true;
        let mut first = true;
        for line in BufReader::new(f).lines() {
            let line = line?;
            if first {
                if !line.contains(MAGIC_HEADER) {
                    // it is NOT one of our generated mod.rs files, so don't modify it!
                    present = true;
                    break;
                }
                first = false;
            }
            if let Some(i) = line.find(matches) {
                let rest = &line[i + matches.len()..line.len() - 1];
                if rest == name {
                    // we already have a reference to this module...
                    present = true;
                    break;
                }
            }
        }
    }
    if !present {
        let mut f = if exists {
            OpenOptions::new().append(true).open(&file)?
        } else {
            let mut f = File::create(&file)?;
            writeln!(f, "{}", MAGIC_HEADER)?;
            f
        };

        writeln!(f, "pub mod {};", name)?;
    }
    Ok(())
}

/// get the proper sanitized file stem from an input file path
fn get_file_stem(path: &Path) -> Result<String> {
    let mut file_stem = path
        .file_stem()
        .and_then(|f| f.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| Error::OutputFile(format!("{}", path.display())))?;

    file_stem = file_stem.replace(|c: char| !c.is_alphanumeric(), "_");
    // will now be properly alphanumeric, but may be a keyword!
    sanitize_keyword(&mut file_stem);
    Ok(file_stem)
}
