//! Runtime tests for generated builders.
//!
//! These tests verify that the generated code not only compiles but also works correctly at runtime. They use `prost`
//! to decode the `piecemeal`-encoded bytes and verify that the decoded values match the original inputs.

use piecemeal::ScratchWriter;
use prost::Message;

use crate::prost_protos;
use crate::protos;

#[test]
fn scalar_types_roundtrip() {
    use prost_protos::scalars::all_scalar_types::AllScalarTypes;
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

    // Decode with prost and verify
    let decoded = AllScalarTypes::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.int32_field, 42);
    assert_eq!(decoded.int64_field, 123456789);
    assert_eq!(decoded.uint32_field, 100);
    assert_eq!(decoded.uint64_field, 200);
    assert_eq!(decoded.sint32_field, -50);
    assert_eq!(decoded.sint64_field, -100);
    assert!(decoded.bool_field);
    assert_eq!(decoded.fixed32_field, 1000);
    assert_eq!(decoded.fixed64_field, 2000);
    assert_eq!(decoded.sfixed32_field, -500);
    assert_eq!(decoded.sfixed64_field, -1000);
    assert_eq!(decoded.float_field, 3.125);
    assert_eq!(decoded.double_field, 2.625);
    assert_eq!(decoded.string_field, "hello");
    assert_eq!(decoded.bytes_field, vec![1, 2, 3, 4]);
}

#[test]
fn enum_roundtrip() {
    use prost_protos::enums::basic_enum::MessageWithEnum;
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

    let decoded = MessageWithEnum::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.status, 1); // ACTIVE = 1
    assert_eq!(decoded.name, "test");
}

#[test]
fn oneof_scalar_variant_roundtrip() {
    use prost_protos::oneofs::basic_oneof::{MessageWithOneof, message_with_oneof::Payload};
    use protos::oneofs::basic_oneof::MessageWithOneofBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MessageWithOneofBuilder::new(&mut scratch_writer);
    builder
        .name("test")
        .unwrap()
        .payload(|p| p.text_value("hello"))
        .unwrap()
        .other_field(42)
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MessageWithOneof::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "test");
    assert_eq!(decoded.other_field, 42);
    assert!(matches!(decoded.payload, Some(Payload::TextValue(ref s)) if s == "hello"));
}

#[test]
fn oneof_int_variant_roundtrip() {
    use prost_protos::oneofs::basic_oneof::{MessageWithOneof, message_with_oneof::Payload};
    use protos::oneofs::basic_oneof::MessageWithOneofBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MessageWithOneofBuilder::new(&mut scratch_writer);
    builder
        .name("test")
        .unwrap()
        .payload(|p| p.int_value(12345))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MessageWithOneof::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "test");
    assert!(matches!(decoded.payload, Some(Payload::IntValue(12345))));
}

#[test]
fn oneof_message_variant_roundtrip() {
    use prost_protos::oneofs::basic_oneof::{MessageWithOneof, message_with_oneof::Payload};
    use protos::oneofs::basic_oneof::MessageWithOneofBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MessageWithOneofBuilder::new(&mut scratch_writer);
    builder
        .name("test")
        .unwrap()
        .payload(|p| {
            p.message_value(|m| {
                m.value("inner")?.count(10)?;
                Ok(())
            })
        })
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MessageWithOneof::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "test");
    match decoded.payload {
        Some(Payload::MessageValue(inner)) => {
            assert_eq!(inner.value, "inner");
            assert_eq!(inner.count, 10);
        }
        _ => panic!("Expected MessageValue variant"),
    }
}

#[test]
fn multiple_oneofs_roundtrip() {
    use prost_protos::oneofs::basic_oneof::{
        MessageWithMultipleOneofs,
        message_with_multiple_oneofs::{FirstChoice, SecondChoice},
    };
    use protos::oneofs::basic_oneof::MessageWithMultipleOneofsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MessageWithMultipleOneofsBuilder::new(&mut scratch_writer);
    builder
        .first_choice(|c| c.option_a("chosen"))
        .unwrap()
        .second_choice(|c| c.amount(123.456))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MessageWithMultipleOneofs::decode(output.as_slice()).unwrap();

    assert!(matches!(decoded.first_choice, Some(FirstChoice::OptionA(ref s)) if s == "chosen"));
    assert!(
        matches!(decoded.second_choice, Some(SecondChoice::Amount(v)) if (v - 123.456).abs() < 0.001)
    );
}

#[test]
fn nested_message_roundtrip() {
    use prost_protos::messages::nested_messages::Outer;
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

    let decoded = Outer::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "outer");
    let middle = decoded.middle.expect("middle should be set");
    assert_eq!(middle.label, "middle");
    let inner = middle.inner.expect("inner should be set");
    assert_eq!(inner.value, "inner_value");
    assert_eq!(inner.count, 42);
}

#[test]
fn repeated_scalars_roundtrip() {
    use prost_protos::repeated::repeated_scalars::RepeatedScalars;
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

    let decoded = RepeatedScalars::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.int32_values, vec![1, 2, 3]);
    assert_eq!(decoded.string_values, vec!["a", "b", "c"]);
    assert_eq!(decoded.double_values, vec![1.0, 2.0, 3.0]);
}

#[test]
fn map_roundtrip() {
    use prost_protos::maps::map_scalar_scalar::MapKeyScalar;
    use protos::maps::map_scalar_scalar::MapKeyScalarBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapKeyScalarBuilder::new(&mut scratch_writer);
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

    let decoded = MapKeyScalar::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.string_to_string.len(), 2);
    assert_eq!(
        decoded.string_to_string.get("key1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        decoded.string_to_string.get("key2"),
        Some(&"value2".to_string())
    );
}

#[test]
fn map_with_message_value_roundtrip() {
    use prost_protos::maps::map_scalar_message::MapKeyMessage;
    use protos::maps::map_scalar_message::MapKeyMessageBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapKeyMessageBuilder::new(&mut scratch_writer);
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

    let decoded = MapKeyMessage::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.string_to_message.len(), 2);
    let entry1 = decoded
        .string_to_message
        .get("key1")
        .expect("key1 should exist");
    assert_eq!(entry1.name, "test_name");
    assert_eq!(entry1.value, 42);
    let entry2 = decoded
        .string_to_message
        .get("key2")
        .expect("key2 should exist");
    assert_eq!(entry2.name, "another_name");
    assert_eq!(entry2.value, 100);
}

#[test]
fn import_roundtrip() {
    use prost_protos::imports::importing_file::ImportingMessage;
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

    let decoded = ImportingMessage::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.extra_field, "extra");
    assert_eq!(decoded.status, 1); // BASE_VALUE_ONE = 1
    let base = decoded.base.expect("base should be set");
    assert_eq!(base.id, "id-123");
    assert_eq!(base.timestamp, 1234567890);
}

#[test]
fn reserved_keywords_roundtrip() {
    use prost_protos::edge_cases::reserved_keywords::KeywordMessage;
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

    let decoded = KeywordMessage::decode(output.as_slice()).unwrap();

    // Prost uses r#keyword syntax for reserved keywords
    assert_eq!(decoded.r#type, "a type");
    assert_eq!(decoded.r#match, 42);
    assert_eq!(decoded.self_, "self value"); // self and crate can't use r# syntax
    assert_eq!(decoded.r#mod, 100);
    assert_eq!(decoded.r#fn, 200);
    assert_eq!(decoded.r#impl, "impl value");
    assert!(!decoded.r#pub);
    assert_eq!(decoded.r#use, 300);
    assert_eq!(decoded.crate_, "crate value"); // self and crate can't use r# syntax
}
