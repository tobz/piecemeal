# AGENTS.md

## Project Overview

Piecemeal is a Rust library for generating Protocol Buffers messages **incrementally**, without requiring the entire message to be in memory before serialization. It generates builder APIs for memory-efficient, streaming-oriented message construction.

**Key constraint:** This crate is serialization-only. It does not support deserializing Protocol Buffers messages.

## Architecture

Rust workspace with three packages:

| Package | Purpose |
|---------|---------|
| `piecemeal/` | Runtime library - traits, wire format writing, scratch buffers |
| `piecemeal-build/` | Code generation - parses `.proto` files, generates builder code |
| `examples/` | Working usage examples |

## Key Files

### Runtime (`piecemeal/src/`)
- `builder.rs` - Generic builders for maps and repeated fields
- `message.rs` - Core traits: `MessageWrite`, `MessageInfo`
- `types.rs` - Wire types and protobuf scalar implementations (macro-heavy)
- `io/writer.rs` - `Writer` trait for encoding operations
- `io/scratch.rs` - `ScratchWriter` for length-prefixed message handling

### Code Generation (`piecemeal-build/src/`)
- `lib.rs` - `ConfigBuilder` entry point
- `parser.rs` - nom-based `.proto` file parser (~1500 lines)
- `types.rs` - Proto AST and code generation logic (~1600 lines)
- `scc.rs` - Cycle detection via strongly connected components

## Development Commands

```bash
cargo build          # Build all packages
cargo test           # Run tests
cargo check          # Type check only
```

## Coding Conventions

- **Builder pattern**: All generated message types use fluent builder APIs
- **Trait abstractions**: `Writer`, `ScratchBuffer`, `MessageWrite`, `ProtobufValue`
- **Macros**: Heavy macro use in `types.rs` for primitive type implementations
- **Documentation**: `#![deny(missing_docs)]` enforced on main crate
- **License**: MIT/Apache-2.0 dual license

## Protobuf Feature Support

**Supported:**
- Non-nested and nested messages
- Repeated fields (streaming serialization)
- Map fields with scalar keys/values
- Proto2 and Proto3 syntax

**Not supported:**
- Map fields with message values
- Enums
- Oneof fields
- Packed repeated fields
- Deserialization

## Generated Code Pattern

Users write `.proto` files, run code generation via `piecemeal-build` in `build.rs`, then use generated builders:

```rust
let mut builder = MessageBuilder::new(&mut scratch_writer);
builder.field_name("value")?
    .add_repeated_field(|nested| {
        nested.inner_field(42)?;
        Ok(())
    })?;
builder.finish(&mut output)?;
```
