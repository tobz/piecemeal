//! Additional documentation and examples.
//!
//! This module contains detailed documentation with runnable examples demonstrating
//! how to use piecemeal to build Protocol Buffers messages incrementally.
//!
//! # Overview
//!
//! Piecemeal generates builder-style APIs from `.proto` files, enabling you to construct
//! Protocol Buffers messages without allocating the entire structure in memory. This is
//! particularly useful for:
//!
//! - Streaming large messages without buffering
//! - Working with borrowed data (`&str` instead of `String`)
//! - Minimizing memory allocations in performance-critical code
//!
//! # Quick Example
//!
//! Given this proto definition:
//!
//! ```proto
//! message Person {
//!     string name = 1;
//!     int32 age = 2;
//! }
//! ```
//!
//! You can build messages like this:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::PersonBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PersonBuilder::new(&mut scratch);
//!
//! builder
//!     .name("Alice")?
//!     .age(30)?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Topics
//!
//! - [`getting_started`] - Basic setup and your first message
//! - [`scalar_fields`] - Working with scalar types and flexible type acceptance
//! - [`nested_messages`] - Building messages within messages
//! - [`repeated_fields`] - Repeated fields and packed encoding
//! - [`maps`] - Map fields with scalar and message values
//! - [`enums`] - Using enum types
//! - [`oneofs`] - Oneof fields and variant selection
//! - [`streaming`] - Incremental serialization patterns

#[doc(hidden)]
#[path = "generated/mod.rs"]
pub mod generated;

pub use generated::blog;
pub use generated::tutorial;

pub mod enums;
pub mod getting_started;
pub mod maps;
pub mod nested_messages;
pub mod oneofs;
pub mod repeated_fields;
pub mod scalar_fields;
pub mod streaming;
