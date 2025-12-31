//! Runtime tests for generated builders.
//!
//! These tests verify that the generated code not only compiles but also
//! works correctly at runtime.

use piecemeal::ScratchWriter;

use crate::protos;

#[test]
fn scalar_types_builder_works() {
    use protos::scalars::all_scalar_types::AllScalarTypesBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = AllScalarTypesBuilder::new(&mut scratch_writer);
    builder
        .int32_field(42)
        .unwrap()
        .int64_field(123456789)
        .unwrap()
        .uint32_field(100)
        .unwrap()
        .uint64_field(200)
        .unwrap()
        .sint32_field(-50)
        .unwrap()
        .sint64_field(-100)
        .unwrap()
        .bool_field(true)
        .unwrap()
        .fixed32_field(1000)
        .unwrap()
        .fixed64_field(2000)
        .unwrap()
        .sfixed32_field(-500)
        .unwrap()
        .sfixed64_field(-1000)
        .unwrap()
        .float_field(3.125)
        .unwrap()
        .double_field(2.625)
        .unwrap()
        .string_field("hello")
        .unwrap()
        .bytes_field(&[1, 2, 3, 4])
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty(), "serialized output should not be empty");
}

#[test]
fn enum_builder_works() {
    use protos::enums::basic_enum::{MessageWithEnumBuilder, Status};

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MessageWithEnumBuilder::new(&mut scratch_writer);
    builder
        .status(Status::ACTIVE)
        .unwrap()
        .name("test")
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn nested_message_builder_works() {
    use protos::messages::nested_messages::OuterBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = OuterBuilder::new(&mut scratch_writer);
    builder
        .name("outer")
        .unwrap()
        .middle(|middle| {
            middle.label("middle")?.inner(|inner| {
                inner.value("inner_value")?.count(42)?;
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn repeated_scalars_builder_works() {
    use protos::repeated::repeated_scalars::RepeatedScalarsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = RepeatedScalarsBuilder::new(&mut scratch_writer);
    builder
        .int32_values(|rb| rb.add_many([1, 2, 3]))
        .unwrap()
        .string_values(|rb| rb.add_many(["a", "b", "c"]))
        .unwrap()
        .double_values(|rb| rb.add_many([1.0, 2.0, 3.0]))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn map_builder_works() {
    use protos::maps::map_scalar_scalar::MapScalarScalarBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapScalarScalarBuilder::new(&mut scratch_writer);
    builder
        .string_to_string()
        .write_entry("key1", "value1")
        .unwrap();
    builder
        .string_to_string()
        .write_entry("key2", "value2")
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn map_with_message_value_builder_works() {
    use protos::maps::map_scalar_message::MapScalarMessageBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapScalarMessageBuilder::new(&mut scratch_writer);
    builder
        .string_to_message()
        .write_entry("key1", |inner| {
            inner.name("test_name")?.value(42)?;
            Ok(())
        })
        .unwrap();
    builder
        .string_to_message()
        .write_entry("key2", |inner| {
            inner.name("another_name")?.value(100)?;
            Ok(())
        })
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn import_builder_works() {
    use protos::imports::base_types::BaseEnum;
    use protos::imports::importing_file::ImportingMessageBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = ImportingMessageBuilder::new(&mut scratch_writer);
    builder
        .extra_field("extra")
        .unwrap()
        .status(BaseEnum::BASE_VALUE_ONE)
        .unwrap()
        .base(|base| {
            base.id("id-123")?.timestamp(1234567890)?;
            Ok(())
        })
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn reserved_keywords_builder_works() {
    use protos::edge_cases::reserved_keywords::KeywordMessageBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = KeywordMessageBuilder::new(&mut scratch_writer);
    // All these field names are Rust keywords - piecemeal escapes them with underscore suffix
    builder
        .type_("a type")
        .unwrap()
        .match_(42)
        .unwrap()
        .self_("self value")
        .unwrap()
        .mod_(100)
        .unwrap()
        .fn_(200)
        .unwrap()
        .impl_("impl value")
        .unwrap()
        .pub_(false)
        .unwrap()
        .use_(300)
        .unwrap()
        .crate_("crate value")
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    assert!(!output.is_empty());
}
