//! Nested messages.
//!
//! Piecemeal handles nested messages using closure-based builders, which allows you to
//! build deeply nested structures without intermediate allocations.
//!
//! # Proto Definition
//!
//! Consider this proto with nested messages:
//!
//! ```proto
//! message Author {
//!     string name = 1;
//!     string email = 2;
//! }
//!
//! message Post {
//!     string title = 1;
//!     Author author = 2;
//! }
//! ```
//!
//! # Building Nested Messages
//!
//! Nested message fields use a closure-based API. The closure receives a builder for
//! the nested message:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder
//!     .title("My First Post")?
//!     .author(|author_builder| {
//!         author_builder
//!             .name("Jane Doe")?
//!             .email("jane@example.com")?;
//!         Ok(())
//!     })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! The closure must return `Ok(())` on success. Any errors are propagated using the
//! `?` operator.
//!
//! # Why Closures?
//!
//! The closure-based design serves several purposes:
//!
//! 1. **Scope management**: The nested builder's lifetime is tied to the closure,
//!    ensuring it's finished before the parent continues.
//!
//! 2. **Memory efficiency**: The nested message is written directly to the scratch
//!    buffer without creating intermediate data structures.
//!
//! 3. **Borrowing flexibility**: You can capture and use borrowed data from the
//!    surrounding scope without lifetime issues.
//!
//! # Nested Within Nested
//!
//! The pattern works for arbitrarily deep nesting:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::GreetingBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = GreetingBuilder::new(&mut scratch);
//!
//! builder
//!     .message("Hello!")?
//!     .sender(|person| {
//!         person
//!             .name("Alice")?
//!             .age(30)?
//!             .email("alice@example.com")?;
//!         Ok(())
//!     })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Conditional Nested Messages
//!
//! Since proto3 doesn't distinguish between "not set" and "set to default values",
//! simply omit the field if you don't want to include it:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let include_author = false;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.title("Anonymous Post")?;
//!
//! if include_author {
//!     builder.author(|author| {
//!         author.name("Someone")?;
//!         Ok(())
//!     })?;
//! }
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
