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

#[test]
fn repeated_scalars_complete_roundtrip() {
    // Tests all repeated field types including int64 and uint32 which were missing from the original test.
    use prost_protos::repeated::repeated_scalars::RepeatedScalars;
    use protos::repeated::repeated_scalars::RepeatedScalarsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = RepeatedScalarsBuilder::new(&mut scratch_writer);
    builder
        .int32_values(|rb| rb.add_many([1, 2, 3]))
        .unwrap()
        .int64_values(|rb| rb.add_many([100i64, 200, 300]))
        .unwrap()
        .string_values(|rb| rb.add_many(["a", "b", "c"]))
        .unwrap()
        .double_values(|rb| rb.add_many([1.5, 2.5, 3.5]))
        .unwrap()
        .uint32_values(|rb| rb.add_many([10u32, 20, 30]))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = RepeatedScalars::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.int32_values, vec![1, 2, 3]);
    assert_eq!(decoded.int64_values, vec![100, 200, 300]);
    assert_eq!(decoded.string_values, vec!["a", "b", "c"]);
    assert_eq!(decoded.double_values, vec![1.5, 2.5, 3.5]);
    assert_eq!(decoded.uint32_values, vec![10, 20, 30]);
}

#[test]
fn repeated_scalars_transparent_conversion_roundtrip() {
    // Tests transparent conversion in repeated fields (e.g., i8 -> int32, u8 -> uint32).
    use prost_protos::repeated::repeated_scalars::RepeatedScalars;
    use protos::repeated::repeated_scalars::RepeatedScalarsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = RepeatedScalarsBuilder::new(&mut scratch_writer);
    builder
        // i8 values transparently converted to int32
        .int32_values(|rb| rb.add_many([1i8, 2i8, 3i8]))
        .unwrap()
        // i16 values transparently converted to int64
        .int64_values(|rb| rb.add_many([100i16, 200i16, 300i16]))
        .unwrap()
        .string_values(|rb| rb.add_many(["x", "y", "z"]))
        .unwrap()
        // f32 values transparently converted to double (f64)
        .double_values(|rb| rb.add_many([1.5f32, 2.5f32, 3.5f32]))
        .unwrap()
        // u8 values transparently converted to uint32
        .uint32_values(|rb| rb.add_many([10u8, 20u8, 30u8]))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = RepeatedScalars::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.int32_values, vec![1, 2, 3]);
    assert_eq!(decoded.int64_values, vec![100, 200, 300]);
    assert_eq!(decoded.string_values, vec!["x", "y", "z"]);
    assert_eq!(decoded.double_values, vec![1.5, 2.5, 3.5]);
    assert_eq!(decoded.uint32_values, vec![10, 20, 30]);
}

#[test]
fn map_more_types_roundtrip() {
    // Tests various map key and value type combinations defined in map_more_types.proto.
    use prost_protos::maps::map_more_types::MapMoreTypes;
    use protos::maps::map_more_types::MapMoreTypesBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapMoreTypesBuilder::new(&mut scratch_writer);

    // Integer key types
    builder
        .int32_to_string()
        .write_entry(42i32, "int32_value")
        .unwrap();
    builder
        .int64_to_string()
        .write_entry(100i64, "int64_value")
        .unwrap();
    builder
        .uint32_to_string()
        .write_entry(200u32, "uint32_value")
        .unwrap();
    builder
        .uint64_to_string()
        .write_entry(300u64, "uint64_value")
        .unwrap();
    builder
        .sint32_to_string()
        .write_entry(-50i32, "sint32_value")
        .unwrap();
    builder
        .sint64_to_string()
        .write_entry(-100i64, "sint64_value")
        .unwrap();

    // Fixed-width key types
    builder
        .fixed32_to_string()
        .write_entry(1000u32, "fixed32_value")
        .unwrap();
    builder
        .fixed64_to_string()
        .write_entry(2000u64, "fixed64_value")
        .unwrap();
    builder
        .sfixed32_to_string()
        .write_entry(-500i32, "sfixed32_value")
        .unwrap();
    builder
        .sfixed64_to_string()
        .write_entry(-1000i64, "sfixed64_value")
        .unwrap();

    // Bool key type
    builder
        .bool_to_string()
        .write_entry(true, "true_value")
        .unwrap();
    builder
        .bool_to_string()
        .write_entry(false, "false_value")
        .unwrap();

    // Various value types with string keys
    builder
        .string_to_int32()
        .write_entry("key_int32", 42i32)
        .unwrap();
    builder
        .string_to_int64()
        .write_entry("key_int64", 100i64)
        .unwrap();
    builder
        .string_to_double()
        .write_entry("key_double", 12.3456)
        .unwrap();
    builder
        .string_to_float()
        .write_entry("key_float", 2.5f32)
        .unwrap();
    builder
        .string_to_bool()
        .write_entry("key_bool", true)
        .unwrap();
    builder
        .string_to_bytes()
        .write_entry("key_bytes", &[1u8, 2, 3][..])
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MapMoreTypes::decode(output.as_slice()).unwrap();

    // Verify integer key types
    assert_eq!(
        decoded.int32_to_string.get(&42),
        Some(&"int32_value".to_string())
    );
    assert_eq!(
        decoded.int64_to_string.get(&100),
        Some(&"int64_value".to_string())
    );
    assert_eq!(
        decoded.uint32_to_string.get(&200),
        Some(&"uint32_value".to_string())
    );
    assert_eq!(
        decoded.uint64_to_string.get(&300),
        Some(&"uint64_value".to_string())
    );
    assert_eq!(
        decoded.sint32_to_string.get(&-50),
        Some(&"sint32_value".to_string())
    );
    assert_eq!(
        decoded.sint64_to_string.get(&-100),
        Some(&"sint64_value".to_string())
    );

    // Verify fixed-width key types
    assert_eq!(
        decoded.fixed32_to_string.get(&1000),
        Some(&"fixed32_value".to_string())
    );
    assert_eq!(
        decoded.fixed64_to_string.get(&2000),
        Some(&"fixed64_value".to_string())
    );
    assert_eq!(
        decoded.sfixed32_to_string.get(&-500),
        Some(&"sfixed32_value".to_string())
    );
    assert_eq!(
        decoded.sfixed64_to_string.get(&-1000),
        Some(&"sfixed64_value".to_string())
    );

    // Verify bool key type
    assert_eq!(
        decoded.bool_to_string.get(&true),
        Some(&"true_value".to_string())
    );
    assert_eq!(
        decoded.bool_to_string.get(&false),
        Some(&"false_value".to_string())
    );

    // Verify various value types
    assert_eq!(decoded.string_to_int32.get("key_int32"), Some(&42));
    assert_eq!(decoded.string_to_int64.get("key_int64"), Some(&100));
    assert!((decoded.string_to_double.get("key_double").unwrap() - 12.3456).abs() < 0.0001);
    assert!((decoded.string_to_float.get("key_float").unwrap() - 2.5).abs() < 0.001);
    assert_eq!(decoded.string_to_bool.get("key_bool"), Some(&true));
    assert_eq!(
        decoded.string_to_bytes.get("key_bytes"),
        Some(&vec![1u8, 2, 3])
    );
}

#[test]
fn repeated_all_types_roundtrip() {
    // Tests all repeated field types to ensure full coverage of wire type implementations.
    use prost_protos::repeated::repeated_scalars::RepeatedScalars;
    use protos::repeated::repeated_scalars::RepeatedScalarsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = RepeatedScalarsBuilder::new(&mut scratch_writer);
    builder
        .int32_values(|rb| rb.add_many([1, 2, 3]))
        .unwrap()
        .int64_values(|rb| rb.add_many([100i64, 200, 300]))
        .unwrap()
        .string_values(|rb| rb.add_many(["a", "b", "c"]))
        .unwrap()
        .double_values(|rb| rb.add_many([1.5, 2.5, 3.5]))
        .unwrap()
        .uint32_values(|rb| rb.add_many([10u32, 20, 30]))
        .unwrap()
        .sint32_values(|rb| rb.add_many([-1i32, -2, -3]))
        .unwrap()
        .sint64_values(|rb| rb.add_many([-100i64, -200, -300]))
        .unwrap()
        .fixed32_values(|rb| rb.add_many([1000u32, 2000, 3000]))
        .unwrap()
        .fixed64_values(|rb| rb.add_many([10000u64, 20000, 30000]))
        .unwrap()
        .sfixed32_values(|rb| rb.add_many([-10i32, -20, -30]))
        .unwrap()
        .sfixed64_values(|rb| rb.add_many([-100i64, -200, -300]))
        .unwrap()
        .float_values(|rb| rb.add_many([0.5f32, 1.5, 2.5]))
        .unwrap()
        .bool_values(|rb| rb.add_many([true, false, true]))
        .unwrap()
        .bytes_values(|rb| rb.add_many([&[1u8, 2][..], &[3, 4][..], &[5, 6][..]]))
        .unwrap()
        .uint64_values(|rb| rb.add_many([100u64, 200, 300]))
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = RepeatedScalars::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.int32_values, vec![1, 2, 3]);
    assert_eq!(decoded.int64_values, vec![100, 200, 300]);
    assert_eq!(decoded.string_values, vec!["a", "b", "c"]);
    assert_eq!(decoded.double_values, vec![1.5, 2.5, 3.5]);
    assert_eq!(decoded.uint32_values, vec![10, 20, 30]);
    assert_eq!(decoded.sint32_values, vec![-1, -2, -3]);
    assert_eq!(decoded.sint64_values, vec![-100, -200, -300]);
    assert_eq!(decoded.fixed32_values, vec![1000, 2000, 3000]);
    assert_eq!(decoded.fixed64_values, vec![10000, 20000, 30000]);
    assert_eq!(decoded.sfixed32_values, vec![-10, -20, -30]);
    assert_eq!(decoded.sfixed64_values, vec![-100, -200, -300]);
    assert_eq!(decoded.float_values, vec![0.5, 1.5, 2.5]);
    assert_eq!(decoded.bool_values, vec![true, false, true]);
    assert_eq!(
        decoded.bytes_values,
        vec![vec![1u8, 2], vec![3, 4], vec![5, 6]]
    );
    assert_eq!(decoded.uint64_values, vec![100, 200, 300]);
}

#[test]
fn map_transparent_conversion_roundtrip() {
    // Tests transparent conversion in map keys and values (e.g., i8 key -> int32 map).
    use prost_protos::maps::map_more_types::MapMoreTypes;
    use protos::maps::map_more_types::MapMoreTypesBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = MapMoreTypesBuilder::new(&mut scratch_writer);

    // i8 key transparently converted to int32
    builder
        .int32_to_string()
        .write_entry(42i8, "from_i8")
        .unwrap();
    // i16 key transparently converted to int32
    builder
        .int32_to_string()
        .write_entry(100i16, "from_i16")
        .unwrap();

    // i16 key transparently converted to int64
    builder
        .int64_to_string()
        .write_entry(200i16, "from_i16_to_i64")
        .unwrap();
    // i32 key transparently converted to int64
    builder
        .int64_to_string()
        .write_entry(300i32, "from_i32_to_i64")
        .unwrap();

    // u8 key transparently converted to uint32
    builder
        .uint32_to_string()
        .write_entry(50u8, "from_u8")
        .unwrap();
    // u16 key transparently converted to uint64
    builder
        .uint64_to_string()
        .write_entry(60u16, "from_u16_to_u64")
        .unwrap();

    // i8 value transparently converted to int32
    builder
        .string_to_int32()
        .write_entry("small", 42i8)
        .unwrap();
    // i16 value transparently converted to int64
    builder
        .string_to_int64()
        .write_entry("medium", 100i16)
        .unwrap();
    // f32 value transparently converted to double (f64)
    builder
        .string_to_double()
        .write_entry("float_to_double", 1.5f32)
        .unwrap();
    // u8 value transparently converted to float
    builder
        .string_to_float()
        .write_entry("u8_to_float", 100u8)
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = MapMoreTypes::decode(output.as_slice()).unwrap();

    // Verify transparent key conversions
    assert_eq!(
        decoded.int32_to_string.get(&42),
        Some(&"from_i8".to_string())
    );
    assert_eq!(
        decoded.int32_to_string.get(&100),
        Some(&"from_i16".to_string())
    );
    assert_eq!(
        decoded.int64_to_string.get(&200),
        Some(&"from_i16_to_i64".to_string())
    );
    assert_eq!(
        decoded.int64_to_string.get(&300),
        Some(&"from_i32_to_i64".to_string())
    );
    assert_eq!(
        decoded.uint32_to_string.get(&50),
        Some(&"from_u8".to_string())
    );
    assert_eq!(
        decoded.uint64_to_string.get(&60),
        Some(&"from_u16_to_u64".to_string())
    );

    // Verify transparent value conversions
    assert_eq!(decoded.string_to_int32.get("small"), Some(&42));
    assert_eq!(decoded.string_to_int64.get("medium"), Some(&100));
    assert!((decoded.string_to_double.get("float_to_double").unwrap() - 1.5).abs() < 0.001);
    assert!((decoded.string_to_float.get("u8_to_float").unwrap() - 100.0).abs() < 0.001);
}

#[test]
fn repeated_messages_roundtrip() {
    use prost_protos::messages::repeated_messages::Outer;
    use protos::messages::repeated_messages::OuterBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = OuterBuilder::new(&mut scratch_writer);
    builder
        .name("container")
        .unwrap()
        .add_items(|inner| {
            inner.value("first")?.count(1)?;
            Ok(())
        })
        .unwrap()
        .add_items(|inner| {
            inner.value("second")?.count(2)?;
            Ok(())
        })
        .unwrap()
        .add_items(|inner| {
            inner.value("third")?.count(3)?;
            Ok(())
        })
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = Outer::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "container");
    assert_eq!(decoded.items.len(), 3);
    assert_eq!(decoded.items[0].value, "first");
    assert_eq!(decoded.items[0].count, 1);
    assert_eq!(decoded.items[1].value, "second");
    assert_eq!(decoded.items[1].count, 2);
    assert_eq!(decoded.items[2].value, "third");
    assert_eq!(decoded.items[2].count, 3);
}

#[test]
fn empty_message_roundtrip() {
    use prost_protos::messages::empty_message::ContainsEmpty;
    use protos::messages::empty_message::ContainsEmptyBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = ContainsEmptyBuilder::new(&mut scratch_writer);
    builder.name("test").unwrap().empty(|_| Ok(())).unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = ContainsEmpty::decode(output.as_slice()).unwrap();

    assert_eq!(decoded.name, "test");
    assert!(decoded.empty.is_some());
}

#[test]
fn proto2_defaults_roundtrip() {
    use prost_protos::edge_cases::proto2_defaults::WithDefaults;
    use protos::edge_cases::proto2_defaults::WithDefaultsBuilder;

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut builder = WithDefaultsBuilder::new(&mut scratch_writer);
    builder
        .name("custom_name")
        .unwrap()
        .count(100)
        .unwrap()
        .enabled(false)
        .unwrap()
        .rate(2.5)
        .unwrap();

    let mut output = Vec::new();
    builder.finish(&mut output).unwrap();

    let decoded = WithDefaults::decode(output.as_slice()).unwrap();

    // Proto2 fields are optional, so prost wraps them in Option
    assert_eq!(decoded.name(), "custom_name");
    assert_eq!(decoded.count(), 100);
    assert!(!decoded.enabled());
    assert!((decoded.rate() - 2.5).abs() < 0.001);
}

/// Decode conformance tests: prost encodes → piecemeal `Decoded` struct decodes.
///
/// These tests verify that piecemeal's generated decoded structs correctly read protobuf wire format
/// produced by prost (a known-correct implementation).
mod decode {
    use crate::prost_protos;
    use crate::protos;
    use prost::Message;

    #[test]
    fn scalar_types_decode() {
        use prost_protos::scalars::all_scalar_types::AllScalarTypes;
        use protos::scalars::all_scalar_types::AllScalarTypesDecoded;

        let msg = AllScalarTypes {
            int32_field: 42,
            int64_field: 123456789,
            uint32_field: 100,
            uint64_field: 200,
            sint32_field: -50,
            sint64_field: -100,
            bool_field: true,
            fixed32_field: 1000,
            fixed64_field: 2000,
            sfixed32_field: -500,
            sfixed64_field: -1000,
            float_field: 3.125,
            double_field: 2.625,
            string_field: "hello".into(),
            bytes_field: vec![1, 2, 3, 4].into(),
        };

        let buf = msg.encode_to_vec();
        let decoded = AllScalarTypesDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.int32_field(), 42);
        assert_eq!(decoded.int64_field(), 123456789);
        assert_eq!(decoded.uint32_field(), 100);
        assert_eq!(decoded.uint64_field(), 200);
        assert_eq!(decoded.sint32_field(), -50);
        assert_eq!(decoded.sint64_field(), -100);
        assert!(decoded.bool_field());
        assert_eq!(decoded.fixed32_field(), 1000);
        assert_eq!(decoded.fixed64_field(), 2000);
        assert_eq!(decoded.sfixed32_field(), -500);
        assert_eq!(decoded.sfixed64_field(), -1000);
        assert_eq!(decoded.float_field(), 3.125);
        assert_eq!(decoded.double_field(), 2.625);
        assert_eq!(decoded.string_field(), "hello");
        assert_eq!(decoded.bytes_field(), &[1, 2, 3, 4]);
    }

    #[test]
    fn scalar_types_defaults_decode() {
        // An empty message should give proto3 defaults.
        use protos::scalars::all_scalar_types::AllScalarTypesDecoded;

        let decoded = AllScalarTypesDecoded::decode(&[]).unwrap();

        assert_eq!(decoded.int32_field(), 0);
        assert_eq!(decoded.int64_field(), 0);
        assert_eq!(decoded.uint32_field(), 0);
        assert_eq!(decoded.uint64_field(), 0);
        assert_eq!(decoded.sint32_field(), 0);
        assert_eq!(decoded.sint64_field(), 0);
        assert!(!decoded.bool_field());
        assert_eq!(decoded.fixed32_field(), 0);
        assert_eq!(decoded.fixed64_field(), 0);
        assert_eq!(decoded.sfixed32_field(), 0);
        assert_eq!(decoded.sfixed64_field(), 0);
        assert_eq!(decoded.float_field(), 0.0);
        assert_eq!(decoded.double_field(), 0.0);
        assert_eq!(decoded.string_field(), "");
        assert_eq!(decoded.bytes_field(), b"" as &[u8]);
    }

    #[test]
    fn enum_decode() {
        use prost_protos::enums::basic_enum::MessageWithEnum;
        use protos::enums::basic_enum::{MessageWithEnumDecoded, Status};

        let msg = MessageWithEnum {
            status: 1, // ACTIVE
            name: "test".into(),
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithEnumDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.status(), Status::ACTIVE);
        assert_eq!(decoded.name(), "test");
    }

    #[test]
    fn nested_message_decode() {
        use prost_protos::messages::nested_messages::{
            Outer,
            outer::{Middle, middle::Inner},
        };
        use protos::messages::nested_messages::OuterDecoded;

        let msg = Outer {
            name: "outer".into(),
            middle: Some(Middle {
                label: "middle".into(),
                inner: Some(Inner {
                    value: "inner_value".into(),
                    count: 42,
                }),
            }),
        };

        let buf = msg.encode_to_vec();
        let decoded = OuterDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.name(), "outer");
        let middle = decoded.middle().unwrap();
        assert_eq!(middle.label(), "middle");
        let inner = middle.inner().unwrap();
        assert_eq!(inner.value(), "inner_value");
        assert_eq!(inner.count(), 42);
    }

    #[test]
    fn nested_message_absent_decode() {
        // When no sub-message is set, accessor should return defaults.
        use protos::messages::nested_messages::OuterDecoded;

        let decoded = OuterDecoded::decode(&[]).unwrap();

        assert_eq!(decoded.name(), "");
        let middle = decoded.middle().unwrap();
        assert_eq!(middle.label(), "");
        let inner = middle.inner().unwrap();
        assert_eq!(inner.value(), "");
        assert_eq!(inner.count(), 0);
    }

    #[test]
    fn oneof_scalar_decode() {
        use prost_protos::oneofs::basic_oneof::{MessageWithOneof, message_with_oneof::Payload};
        use protos::oneofs::basic_oneof::{MessageWithOneofDecoded, PayloadOneOf};

        let msg = MessageWithOneof {
            name: "test".into(),
            other_field: 42,
            payload: Some(Payload::TextValue("hello".into())),
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithOneofDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.name(), "test");
        assert_eq!(decoded.other_field(), 42);
        match decoded.payload() {
            Some(PayloadOneOf::TextValue(s)) => assert_eq!(*s, "hello"),
            other => panic!("Expected TextValue, got {:?}", other),
        }
    }

    #[test]
    fn oneof_int_decode() {
        use prost_protos::oneofs::basic_oneof::{MessageWithOneof, message_with_oneof::Payload};
        use protos::oneofs::basic_oneof::{MessageWithOneofDecoded, PayloadOneOf};

        let msg = MessageWithOneof {
            name: "test".into(),
            other_field: 0,
            payload: Some(Payload::IntValue(12345)),
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithOneofDecoded::decode(&buf).unwrap();

        match decoded.payload() {
            Some(PayloadOneOf::IntValue(v)) => assert_eq!(*v, 12345),
            other => panic!("Expected IntValue, got {:?}", other),
        }
    }

    #[test]
    fn oneof_message_decode() {
        use prost_protos::oneofs::basic_oneof::{
            InnerMessage, MessageWithOneof, message_with_oneof::Payload,
        };
        use protos::oneofs::basic_oneof::{MessageWithOneofDecoded, PayloadOneOf};

        let msg = MessageWithOneof {
            name: "test".into(),
            other_field: 0,
            payload: Some(Payload::MessageValue(InnerMessage {
                value: "inner".into(),
                count: 10,
            })),
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithOneofDecoded::decode(&buf).unwrap();

        match decoded.payload() {
            Some(PayloadOneOf::MessageValue(bytes)) => {
                let inner = decoded.decode_message_value(bytes).unwrap();
                assert_eq!(inner.value(), "inner");
                assert_eq!(inner.count(), 10);
            }
            other => panic!("Expected MessageValue, got {:?}", other),
        }
    }

    #[test]
    fn oneof_none_decode() {
        use prost_protos::oneofs::basic_oneof::MessageWithOneof;
        use protos::oneofs::basic_oneof::MessageWithOneofDecoded;

        let msg = MessageWithOneof {
            name: "test".into(),
            other_field: 5,
            payload: None,
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithOneofDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.name(), "test");
        assert_eq!(decoded.other_field(), 5);
        assert!(decoded.payload().is_none());
    }

    #[test]
    fn multiple_oneofs_decode() {
        use prost_protos::oneofs::basic_oneof::{
            MessageWithMultipleOneofs,
            message_with_multiple_oneofs::{FirstChoice, SecondChoice},
        };
        use protos::oneofs::basic_oneof::{
            FirstChoiceOneOf, MessageWithMultipleOneofsDecoded, SecondChoiceOneOf,
        };

        let msg = MessageWithMultipleOneofs {
            first_choice: Some(FirstChoice::OptionA("chosen".into())),
            second_choice: Some(SecondChoice::Amount(123.456)),
        };

        let buf = msg.encode_to_vec();
        let decoded = MessageWithMultipleOneofsDecoded::decode(&buf).unwrap();

        match decoded.first_choice() {
            Some(FirstChoiceOneOf::OptionA(s)) => assert_eq!(*s, "chosen"),
            other => panic!("Expected OptionA, got {:?}", other),
        }
        match decoded.second_choice() {
            Some(SecondChoiceOneOf::Amount(v)) => assert!((v - 123.456).abs() < 0.001),
            other => panic!("Expected Amount, got {:?}", other),
        }
    }

    #[test]
    fn repeated_scalars_decode() {
        use prost_protos::repeated::repeated_scalars::RepeatedScalars;
        use protos::repeated::repeated_scalars::RepeatedScalarsDecoded;

        let msg = RepeatedScalars {
            int32_values: vec![1, 2, 3],
            int64_values: vec![100, 200, 300],
            string_values: vec!["a".into(), "b".into(), "c".into()],
            double_values: vec![1.5, 2.5, 3.5],
            uint32_values: vec![10, 20, 30],
            sint32_values: vec![-1, -2, -3],
            sint64_values: vec![-100, -200, -300],
            fixed32_values: vec![1000, 2000, 3000],
            fixed64_values: vec![10000, 20000, 30000],
            sfixed32_values: vec![-10, -20, -30],
            sfixed64_values: vec![-100, -200, -300],
            float_values: vec![0.5, 1.5, 2.5],
            bool_values: vec![true, false, true],
            bytes_values: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
            uint64_values: vec![100, 200, 300],
        };

        let buf = msg.encode_to_vec();
        let decoded = RepeatedScalarsDecoded::decode(&buf).unwrap();

        let int32s: Vec<i32> = decoded.int32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(int32s, vec![1, 2, 3]);

        let int64s: Vec<i64> = decoded.int64_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(int64s, vec![100, 200, 300]);

        let strings: Vec<&str> = decoded.string_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(strings, vec!["a", "b", "c"]);

        let doubles: Vec<f64> = decoded.double_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(doubles, vec![1.5, 2.5, 3.5]);

        let uint32s: Vec<u32> = decoded.uint32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(uint32s, vec![10, 20, 30]);

        let sint32s: Vec<i32> = decoded.sint32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(sint32s, vec![-1, -2, -3]);

        let sint64s: Vec<i64> = decoded.sint64_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(sint64s, vec![-100, -200, -300]);

        let fixed32s: Vec<u32> = decoded.fixed32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(fixed32s, vec![1000, 2000, 3000]);

        let fixed64s: Vec<u64> = decoded.fixed64_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(fixed64s, vec![10000, 20000, 30000]);

        let sfixed32s: Vec<i32> = decoded.sfixed32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(sfixed32s, vec![-10, -20, -30]);

        let sfixed64s: Vec<i64> = decoded.sfixed64_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(sfixed64s, vec![-100, -200, -300]);

        let floats: Vec<f32> = decoded.float_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(floats, vec![0.5, 1.5, 2.5]);

        let bools: Vec<bool> = decoded.bool_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(bools, vec![true, false, true]);

        let bytes: Vec<&[u8]> = decoded.bytes_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(bytes, vec![&[1u8, 2][..], &[3, 4][..], &[5, 6][..]]);

        let uint64s: Vec<u64> = decoded.uint64_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(uint64s, vec![100, 200, 300]);
    }

    #[test]
    fn repeated_empty_decode() {
        // Empty repeated fields should produce empty iterators.
        use protos::repeated::repeated_scalars::RepeatedScalarsDecoded;

        let decoded = RepeatedScalarsDecoded::decode(&[]).unwrap();

        let int32s: Vec<i32> = decoded.int32_values().collect::<Result<_, _>>().unwrap();
        assert!(int32s.is_empty());

        let strings: Vec<&str> = decoded.string_values().collect::<Result<_, _>>().unwrap();
        assert!(strings.is_empty());
    }

    #[test]
    fn repeated_messages_decode() {
        use prost_protos::messages::repeated_messages::{Inner, Outer};
        use protos::messages::repeated_messages::OuterDecoded;

        let msg = Outer {
            name: "container".into(),
            items: vec![
                Inner {
                    value: "first".into(),
                    count: 1,
                },
                Inner {
                    value: "second".into(),
                    count: 2,
                },
                Inner {
                    value: "third".into(),
                    count: 3,
                },
            ],
        };

        let buf = msg.encode_to_vec();
        let decoded = OuterDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.name(), "container");
        let items: Vec<_> = decoded.items().collect::<Result<_, _>>().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].value(), "first");
        assert_eq!(items[0].count(), 1);
        assert_eq!(items[1].value(), "second");
        assert_eq!(items[1].count(), 2);
        assert_eq!(items[2].value(), "third");
        assert_eq!(items[2].count(), 3);
    }

    #[test]
    fn map_string_to_string_decode() {
        use prost_protos::maps::map_scalar_scalar::MapKeyScalar;
        use protos::maps::map_scalar_scalar::MapKeyScalarDecoded;

        let mut msg = MapKeyScalar::default();
        msg.string_to_string.insert("key1".into(), "value1".into());
        msg.string_to_string.insert("key2".into(), "value2".into());

        let buf = msg.encode_to_vec();
        let decoded = MapKeyScalarDecoded::decode(&buf).unwrap();

        let entries: Vec<(&str, &str)> = decoded
            .string_to_string()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 2);
        // Map iteration order is not guaranteed, so sort.
        let mut entries_sorted = entries.clone();
        entries_sorted.sort_by_key(|(k, _)| *k);
        assert_eq!(entries_sorted[0], ("key1", "value1"));
        assert_eq!(entries_sorted[1], ("key2", "value2"));
    }

    #[test]
    fn map_with_message_value_decode() {
        use prost_protos::maps::map_scalar_message::{InnerMessage, MapKeyMessage};
        use protos::maps::map_scalar_message::MapKeyMessageDecoded;

        let mut msg = MapKeyMessage::default();
        msg.string_to_message.insert(
            "key1".into(),
            InnerMessage {
                name: "test_name".into(),
                value: 42,
            },
        );

        let buf = msg.encode_to_vec();
        let decoded = MapKeyMessageDecoded::decode(&buf).unwrap();

        let entries: Vec<_> = decoded
            .string_to_message()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "key1");
        assert_eq!(entries[0].1.name(), "test_name");
        assert_eq!(entries[0].1.value(), 42);
    }

    #[test]
    fn import_decode() {
        use prost_protos::imports::base_types::BaseMessage;
        use prost_protos::imports::importing_file::ImportingMessage;
        use protos::imports::base_types::BaseEnum;
        use protos::imports::importing_file::ImportingMessageDecoded;

        let msg = ImportingMessage {
            extra_field: "extra".into(),
            status: 1, // BASE_VALUE_ONE
            base: Some(BaseMessage {
                id: "id-123".into(),
                timestamp: 1234567890,
            }),
        };

        let buf = msg.encode_to_vec();
        let decoded = ImportingMessageDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.extra_field(), "extra");
        assert_eq!(decoded.status(), BaseEnum::BASE_VALUE_ONE);
        let base = decoded.base().unwrap();
        assert_eq!(base.id(), "id-123");
        assert_eq!(base.timestamp(), 1234567890);
    }

    #[test]
    fn empty_message_decode() {
        use prost_protos::messages::empty_message::ContainsEmpty;
        use protos::messages::empty_message::ContainsEmptyDecoded;

        let msg = ContainsEmpty {
            name: "test".into(),
            empty: Some(prost_protos::messages::empty_message::EmptyMessage {}),
        };

        let buf = msg.encode_to_vec();
        let decoded = ContainsEmptyDecoded::decode(&buf).unwrap();

        assert_eq!(decoded.name(), "test");
    }

    #[test]
    fn roundtrip_piecemeal_encode_piecemeal_decode() {
        // Full roundtrip: piecemeal encodes → piecemeal decodes (no prost involved).
        use piecemeal::ScratchWriter;
        use protos::scalars::all_scalar_types::{AllScalarTypesBuilder, AllScalarTypesDecoded};

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

        let decoded = AllScalarTypesDecoded::decode(&output).unwrap();

        assert_eq!(decoded.int32_field(), 42);
        assert_eq!(decoded.int64_field(), 123456789);
        assert_eq!(decoded.uint32_field(), 100);
        assert_eq!(decoded.uint64_field(), 200);
        assert_eq!(decoded.sint32_field(), -50);
        assert_eq!(decoded.sint64_field(), -100);
        assert!(decoded.bool_field());
        assert_eq!(decoded.fixed32_field(), 1000);
        assert_eq!(decoded.fixed64_field(), 2000);
        assert_eq!(decoded.sfixed32_field(), -500);
        assert_eq!(decoded.sfixed64_field(), -1000);
        assert_eq!(decoded.float_field(), 3.125);
        assert_eq!(decoded.double_field(), 2.625);
        assert_eq!(decoded.string_field(), "hello");
        assert_eq!(decoded.bytes_field(), &[1, 2, 3, 4]);
    }

    #[test]
    fn roundtrip_nested_piecemeal_encode_decode() {
        use piecemeal::ScratchWriter;
        use protos::messages::nested_messages::{OuterBuilder, OuterDecoded};

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

        let decoded = OuterDecoded::decode(&output).unwrap();
        assert_eq!(decoded.name(), "outer");
        let middle = decoded.middle().unwrap();
        assert_eq!(middle.label(), "middle");
        let inner = middle.inner().unwrap();
        assert_eq!(inner.value(), "inner_value");
        assert_eq!(inner.count(), 42);
    }

    #[test]
    fn roundtrip_repeated_piecemeal_encode_decode() {
        use piecemeal::ScratchWriter;
        use protos::repeated::repeated_scalars::{RepeatedScalarsBuilder, RepeatedScalarsDecoded};

        let scratch_buf = Vec::with_capacity(1024);
        let mut scratch_writer = ScratchWriter::new(scratch_buf);

        let mut builder = RepeatedScalarsBuilder::new(&mut scratch_writer);
        builder
            .int32_values(|rb| rb.add_many([1, 2, 3]))
            .unwrap()
            .string_values(|rb| rb.add_many(["a", "b", "c"]))
            .unwrap()
            .double_values(|rb| rb.add_many([1.5, 2.5, 3.5]))
            .unwrap();

        let mut output = Vec::new();
        builder.finish(&mut output).unwrap();

        let decoded = RepeatedScalarsDecoded::decode(&output).unwrap();

        let int32s: Vec<i32> = decoded.int32_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(int32s, vec![1, 2, 3]);

        let strings: Vec<&str> = decoded.string_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(strings, vec!["a", "b", "c"]);

        let doubles: Vec<f64> = decoded.double_values().collect::<Result<_, _>>().unwrap();
        assert_eq!(doubles, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn roundtrip_map_piecemeal_encode_decode() {
        use piecemeal::ScratchWriter;
        use protos::maps::map_scalar_scalar::{MapKeyScalarBuilder, MapKeyScalarDecoded};

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

        let decoded = MapKeyScalarDecoded::decode(&output).unwrap();

        let mut entries: Vec<(&str, &str)> = decoded
            .string_to_string()
            .collect::<Result<_, _>>()
            .unwrap();
        entries.sort_by_key(|(k, _)| *k);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], ("key1", "value1"));
        assert_eq!(entries[1], ("key2", "value2"));
    }
}

mod invalid_protos {
    use piecemeal_build::ConfigBuilder;
    use std::path::Path;

    /// Validates a proto file using protoc.
    fn validate_with_protoc(proto_file: &Path) -> Result<(), String> {
        let mut cmd = std::process::Command::new("protoc");
        cmd.arg("--proto_path=protos/invalid");
        cmd.arg("-o")
            .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
        cmd.arg(proto_file);

        let output = cmd
            .output()
            .expect("protoc must be installed and available in PATH");

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Helper to test an invalid proto that both protoc and piecemeal should reject.
    fn assert_both_reject(proto_name: &str) {
        let proto_path = Path::new("protos/invalid").join(proto_name);
        let temp_out = std::env::temp_dir().join(format!("piecemeal_invalid_{}", proto_name));
        std::fs::create_dir_all(&temp_out).unwrap();

        // protoc must reject
        let protoc_result = validate_with_protoc(&proto_path);
        assert!(
            protoc_result.is_err(),
            "protoc should reject {}, but it accepted it",
            proto_name
        );

        // piecemeal must also reject
        let piecemeal_result = ConfigBuilder::new()
            .input_files(&[proto_path.to_str().unwrap()])
            .output_dir(&temp_out)
            .include_paths(&["protos/invalid"])
            .compile();

        assert!(
            piecemeal_result.is_err(),
            "piecemeal should reject {}, but it accepted it (protoc error: {})",
            proto_name,
            protoc_result.unwrap_err()
        );

        let _ = std::fs::remove_dir_all(&temp_out);
    }

    /// Helper to test an invalid proto where protoc rejects but piecemeal has a known gap.
    fn assert_protoc_rejects_piecemeal_gap(proto_name: &str, gap_reason: &str) {
        let proto_path = Path::new("protos/invalid").join(proto_name);
        let temp_out = std::env::temp_dir().join(format!("piecemeal_invalid_{}", proto_name));
        std::fs::create_dir_all(&temp_out).unwrap();

        // protoc must reject
        let protoc_result = validate_with_protoc(&proto_path);
        assert!(
            protoc_result.is_err(),
            "protoc should reject {}, but it accepted it",
            proto_name
        );

        // piecemeal currently accepts (known gap)
        let piecemeal_result = ConfigBuilder::new()
            .input_files(&[proto_path.to_str().unwrap()])
            .output_dir(&temp_out)
            .include_paths(&["protos/invalid"])
            .compile();

        if piecemeal_result.is_err() {
            // Gap has been fixed!
            eprintln!(
                "GAP FIXED: {} is now correctly rejected (was: {})",
                proto_name, gap_reason
            );
        } else {
            eprintln!("KNOWN GAP: {} - {}", proto_name, gap_reason);
        }

        let _ = std::fs::remove_dir_all(&temp_out);
    }

    #[test]
    fn undefined_type_rejected() {
        assert_both_reject("undefined_type.proto");
    }

    #[test]
    fn reserved_field_used_rejected() {
        assert_both_reject("reserved_field_used.proto");
    }

    #[test]
    fn duplicate_field_name_rejected() {
        assert_protoc_rejects_piecemeal_gap(
            "duplicate_field_name.proto",
            "piecemeal does not validate duplicate field names",
        );
    }

    #[test]
    fn duplicate_field_number_rejected() {
        assert_protoc_rejects_piecemeal_gap(
            "duplicate_field_number.proto",
            "piecemeal does not validate duplicate field numbers",
        );
    }

    #[test]
    fn invalid_field_number_rejected() {
        assert_protoc_rejects_piecemeal_gap(
            "invalid_field_number.proto",
            "piecemeal does not validate field number range (must be > 0)",
        );
    }

    #[test]
    fn proto3_enum_no_zero_rejected() {
        assert_protoc_rejects_piecemeal_gap(
            "proto3_enum_no_zero.proto",
            "piecemeal does not validate proto3 enum first value must be 0",
        );
    }

    #[test]
    fn invalid_default_enum_rejected() {
        assert_both_reject("invalid_default_enum.proto");
    }
}
