//! Getting started with piecemeal.
//!
//! This guide walks you through the basics of using piecemeal to build Protocol Buffers
//! messages.
//!
//! # Setup
//!
//! Add piecemeal to your project:
//!
//! ```toml
//! [dependencies]
//! piecemeal = "0.1"
//!
//! [build-dependencies]
//! piecemeal-build = "0.1"
//! ```
//!
//! Create a `build.rs` file to compile your `.proto` files:
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     piecemeal_build::ConfigBuilder::new()
//!         .input_files(&["./protos/messages.proto"])
//!         .cargo_output_dir("protos")
//!         .unwrap()
//!         .include_paths(&["./protos"])
//!         .compile()
//!         .unwrap();
//! }
//! ```
//!
//! # Core Concepts
//!
//! ## The ScratchWriter
//!
//! Every piecemeal builder requires a [`ScratchWriter`]. This is a temporary buffer used
//! to handle length-delimited fields in Protocol Buffers. Because the length of a message
//! must be written *before* its contents, piecemeal uses the scratch buffer to write
//! the message first, calculate its length, and then copy it to the final output.
//!
//! ```
//! use piecemeal::ScratchWriter;
//!
//! // Create a scratch writer with an initial buffer
//! let mut scratch = ScratchWriter::new(Vec::new());
//! ```
//!
//! The scratch writer is reused across multiple message builds, so you typically create
//! it once and pass it to each builder.
//!
//! ## Building a Message
//!
//! Given this proto definition:
//!
//! ```proto
//! message Person {
//!     string name = 1;
//!     int32 age = 2;
//!     string email = 3;
//! }
//! ```
//!
//! You build a message by:
//!
//! 1. Creating a builder from the scratch writer
//! 2. Setting fields using the builder methods
//! 3. Calling `finish()` to write the final bytes
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::PersonBuilder;
//!
//! // Step 1: Create the scratch writer and builder
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PersonBuilder::new(&mut scratch);
//!
//! // Step 2: Set fields (order doesn't matter in proto3)
//! builder
//!     .name("Alice")?
//!     .age(30)?
//!     .email("alice@example.com")?;
//!
//! // Step 3: Finish and get the encoded bytes
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//!
//! // `output` now contains the serialized Protocol Buffers message
//! assert!(!output.is_empty());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Builder Method Patterns
//!
//! Piecemeal generates different method signatures based on the field type:
//!
//! - **Scalar fields** (e.g., `int32`, `string`): Direct methods like `.name("value")`
//! - **Nested messages**: Closure-based methods like `.sender(|b| { ... })`
//! - **Repeated fields**: Methods prefixed with `add_` like `.add_tags(|b| { ... })`
//! - **Map fields**: Return a map builder with `.write_entry()` method
//!
//! All builder methods return `io::Result<&mut Self>`, allowing you to chain calls
//! with the `?` operator.
//!
//! ## Finishing a Message
//!
//! The `finish()` method consumes the builder and writes the complete message to
//! the provided output:
//!
//! ```
//! # use piecemeal::ScratchWriter;
//! # use piecemeal::docs::tutorial::PersonBuilder;
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PersonBuilder::new(&mut scratch);
//! # builder.name("Alice")?;
//! // Write to a Vec<u8>
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! For length-delimited encoding (common when embedding messages), use
//! `finish_length_delimited()`:
//!
//! ```
//! # use piecemeal::ScratchWriter;
//! # use piecemeal::docs::tutorial::PersonBuilder;
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PersonBuilder::new(&mut scratch);
//! # builder.name("Alice")?;
//! let mut output = Vec::new();
//! builder.finish_length_delimited(&mut output)?;
//! // Output now has a varint length prefix followed by the message
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! [`ScratchWriter`]: crate::ScratchWriter
