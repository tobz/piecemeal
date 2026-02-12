//! Repeated fields and packed encoding.
//!
//! Piecemeal supports repeated fields for both scalar and message types, with automatic
//! packed encoding for scalar types in proto3.
//!
//! # Proto Definition
//!
//! ```proto
//! message Post {
//!     repeated string tags = 4;
//!     repeated Comment comments = 5;
//! }
//! ```
//!
//! # Repeated Scalar Fields
//!
//! Repeated scalar fields use the `add_<field>` pattern with a closure that receives
//! a [`RepeatedBuilder`]:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.tags(|tags| {
//!     tags.add("rust")?;
//!     tags.add("protobuf")?;
//!     tags.add("serialization")?;
//!     Ok(())
//! })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Adding Multiple Values at Once
//!
//! Use `add_many()` to add multiple values from an iterator:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! let my_tags = vec!["rust", "protobuf", "serialization"];
//!
//! builder.tags(|tags| {
//!     tags.add_many(my_tags.iter().copied())?;
//!     Ok(())
//! })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! You can also use array literals directly:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.tags(|tags| tags.add_many(["rust", "protobuf"]))?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Repeated Message Fields
//!
//! Repeated message fields use `add_<field>` with a closure for each message:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder
//!     .title("My Post")?
//!     .add_comments(|comment| {
//!         comment
//!             .author_name("Alice")?
//!             .content("Great post!")?
//!             .timestamp(1704067200)?;
//!         Ok(())
//!     })?
//!     .add_comments(|comment| {
//!         comment
//!             .author_name("Bob")?
//!             .content("Thanks for sharing.")?
//!             .timestamp(1704070800)?;
//!         Ok(())
//!     })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Each call to `add_comments()` adds one message to the repeated field. Call it
//! multiple times to add multiple messages.
//!
//! # Adding Messages from a Collection
//!
//! To add messages from a collection, iterate and call the add method for each:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! struct CommentData {
//!     author: &'static str,
//!     text: &'static str,
//!     time: i64,
//! }
//!
//! let comments = vec![
//!     CommentData { author: "Alice", text: "Great!", time: 1000 },
//!     CommentData { author: "Bob", text: "Thanks!", time: 2000 },
//! ];
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//! builder.title("My Post")?;
//!
//! for c in &comments {
//!     builder.add_comments(|comment| {
//!         comment
//!             .author_name(c.author)?
//!             .content(c.text)?
//!             .timestamp(c.time)?;
//!         Ok(())
//!     })?;
//! }
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Packed Encoding
//!
//! In proto3, repeated scalar fields (except `string` and `bytes`) use packed encoding
//! by default. This is more efficient on the wire because the field tag is only written
//! once, followed by the total length and all values.
//!
//! Piecemeal handles packed encoding automatically. When you use `add_many()` on a
//! packable type, the values are written in packed format:
//!
//! ```ignore
//! // For a repeated int32 field:
//! builder.values(|vals| {
//!     vals.add_many([1, 2, 3, 4, 5])?;  // Written as a single packed field
//!     Ok(())
//! })?;
//! ```
//!
//! Individual `add()` calls write each value as a separate field, which is valid but
//! less efficient. Prefer `add_many()` when you have multiple values available.
//!
//! [`RepeatedBuilder`]: crate::RepeatedBuilder
