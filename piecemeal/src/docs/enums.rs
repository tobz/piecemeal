//! Enum types.
//!
//! Piecemeal generates Rust enums for Protocol Buffers enum types, providing type-safe
//! usage in your code.
//!
//! # Proto Definition
//!
//! ```proto
//! enum PostStatus {
//!     DRAFT = 0;
//!     PUBLISHED = 1;
//!     ARCHIVED = 2;
//! }
//!
//! message Post {
//!     PostStatus status = 7;
//! }
//! ```
//!
//! # Generated Enum
//!
//! Piecemeal generates a Rust enum with the same variants:
//!
//! ```ignore
//! #[derive(Debug, PartialEq, Eq, Clone, Copy)]
//! pub enum PostStatus {
//!     DRAFT = 0,
//!     PUBLISHED = 1,
//!     ARCHIVED = 2,
//! }
//! ```
//!
//! # Using Enums
//!
//! Use the generated enum directly when setting enum fields:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::{PostBuilder, PostStatus};
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder
//!     .title("My Draft Post")?
//!     .status(PostStatus::DRAFT)?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Enum Conversions
//!
//! The generated enums implement `From<i32>` for converting from integer values,
//! which is useful when working with dynamic data:
//!
//! ```
//! use piecemeal::docs::blog::PostStatus;
//!
//! // Convert from i32
//! let status: PostStatus = PostStatus::from(1);
//! assert_eq!(status, PostStatus::PUBLISHED);
//!
//! // Unknown values map to the first variant (usually the zero value)
//! let unknown: PostStatus = PostStatus::from(999);
//! assert_eq!(unknown, PostStatus::DRAFT);  // Falls back to 0
//! ```
//!
//! Enums also implement `From<&str>` for converting from string names:
//!
//! ```
//! use piecemeal::docs::blog::PostStatus;
//!
//! let status: PostStatus = PostStatus::from("PUBLISHED");
//! assert_eq!(status, PostStatus::PUBLISHED);
//!
//! // Unknown names map to the first variant
//! let unknown: PostStatus = PostStatus::from("UNKNOWN_VALUE");
//! assert_eq!(unknown, PostStatus::DRAFT);
//! ```
//!
//! # The Zero Value
//!
//! In proto3, every enum must have a variant with value 0, and this is the default
//! value. When you don't set an enum field, it defaults to this zero variant.
//!
//! Convention is to name this variant something like `UNKNOWN` or `UNSPECIFIED`:
//!
//! ```proto
//! enum Status {
//!     STATUS_UNSPECIFIED = 0;
//!     STATUS_ACTIVE = 1;
//!     STATUS_INACTIVE = 2;
//! }
//! ```
//!
//! # Using Enums with Match
//!
//! Since the generated enums derive `Copy` and `Clone`, you can use them in match
//! expressions and pass them around freely:
//!
//! ```
//! use piecemeal::docs::blog::PostStatus;
//!
//! fn describe_status(status: PostStatus) -> &'static str {
//!     match status {
//!         PostStatus::DRAFT => "This post is a draft",
//!         PostStatus::PUBLISHED => "This post is live",
//!         PostStatus::ARCHIVED => "This post has been archived",
//!     }
//! }
//!
//! assert_eq!(describe_status(PostStatus::PUBLISHED), "This post is live");
//! ```
