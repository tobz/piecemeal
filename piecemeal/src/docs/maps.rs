//! Map fields.
//!
//! Piecemeal supports Protocol Buffers map fields with both scalar and message values.
//!
//! # Proto Definition
//!
//! ```proto
//! message Post {
//!     map<string, string> metadata = 6;
//! }
//! ```
//!
//! # Scalar-to-Scalar Maps
//!
//! For maps with scalar keys and values, the builder method returns a [`GenericMapBuilder`]
//! that you use to write entries:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! // Get the map builder and write entries
//! let mut metadata = builder.metadata();
//! metadata.write_entry("author", "Jane Doe")?;
//! metadata.write_entry("category", "Technology")?;
//! metadata.write_entry("language", "en")?;
//! drop(metadata);  // Explicitly drop to release borrow
//!
//! builder.title("My Post")?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Note that the map builder borrows the parent builder mutably, so you need to
//! either drop it explicitly or let it go out of scope before continuing with
//! the parent builder.
//!
//! # Using Scopes for Map Builders
//!
//! A cleaner pattern is to use a block scope:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.title("My Post")?;
//!
//! {
//!     let mut metadata = builder.metadata();
//!     metadata.write_entry("author", "Jane Doe")?;
//!     metadata.write_entry("category", "Technology")?;
//! }  // Map builder dropped here, releasing the borrow
//!
//! // Can now continue using builder
//! builder.content("Post content here...")?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Adding Entries from a Collection
//!
//! You can iterate over any collection to add map entries:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//! use std::collections::HashMap;
//!
//! let mut meta: HashMap<&str, &str> = HashMap::new();
//! meta.insert("author", "Jane");
//! meta.insert("version", "1.0");
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! {
//!     let mut metadata = builder.metadata();
//!     for (key, value) in meta {
//!         metadata.write_entry(key, value)?;
//!     }
//! }
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Valid Map Key Types
//!
//! Protocol Buffers restricts map keys to certain types. The following are valid:
//!
//! - `bool`
//! - `int32`, `int64`, `uint32`, `uint64`
//! - `sint32`, `sint64`
//! - `fixed32`, `fixed64`, `sfixed32`, `sfixed64`
//! - `string`
//!
//! Notably, `float`, `double`, and `bytes` cannot be map keys.
//!
//! # Scalar-to-Message Maps
//!
//! For maps where the value is a message type, the builder uses a closure-based API
//! similar to nested messages:
//!
//! ```ignore
//! // For a map<string, Author> field:
//! builder.authors(|authors| {
//!     authors.write_entry("primary", |author| {
//!         author.name("Jane Doe")?;
//!         author.email("jane@example.com")?;
//!         Ok(())
//!     })?;
//!     Ok(())
//! })?;
//! ```
//!
//! This allows you to build the message value inline without intermediate allocations.
//!
//! [`GenericMapBuilder`]: crate::GenericMapBuilder
