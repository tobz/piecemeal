//! Oneof fields.
//!
//! Piecemeal supports Protocol Buffers oneof fields, which represent mutually exclusive
//! options where only one field can be set at a time.
//!
//! # Proto Definition
//!
//! ```proto
//! message Post {
//!     oneof featured_media {
//!         string image_url = 8;
//!         string video_url = 9;
//!     }
//! }
//! ```
//!
//! # Setting Oneof Fields
//!
//! Oneof fields use a closure-based API that provides a specialized builder with methods
//! for each variant:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder
//!     .title("Post with Image")?
//!     .featured_media(|media| {
//!         media.image_url("https://example.com/image.jpg")
//!     })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Or choose a different variant:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder
//!     .title("Post with Video")?
//!     .featured_media(|media| {
//!         media.video_url("https://example.com/video.mp4")
//!     })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Oneof with Message Variants
//!
//! When a oneof contains message types, each variant method takes a closure:
//!
//! ```ignore
//! // For a oneof like:
//! // oneof content {
//! //     TextContent text = 1;
//! //     ImageContent image = 2;
//! // }
//!
//! builder.content(|content| {
//!     content.text(|text_builder| {
//!         text_builder.body("Hello, world!")?;
//!         Ok(())
//!     })
//! })?;
//! ```
//!
//! # Conditional Variant Selection
//!
//! Since you have control flow within the closure, you can choose variants dynamically:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! enum MediaType {
//!     Image(String),
//!     Video(String),
//! }
//!
//! let media = MediaType::Image("https://example.com/photo.jpg".to_string());
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.title("Dynamic Media Post")?;
//!
//! builder.featured_media(|m| {
//!     match &media {
//!         MediaType::Image(url) => m.image_url(url),
//!         MediaType::Video(url) => m.video_url(url),
//!     }
//! })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Omitting Oneof Fields
//!
//! Oneofs are optional. If you don't call the oneof method, no variant is set:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! // No featured_media set
//! builder.title("Text-only Post")?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Important: Only One Variant
//!
//! Protocol Buffers oneofs are designed so only one variant can be set. If you call
//! multiple variant methods within the oneof closure, all will be written to the wire,
//! but decoders will only see the last one (per the protobuf spec). This is valid but
//! usually not what you want:
//!
//! ```ignore
//! // Don't do this - both get written but decoder sees only video_url
//! builder.featured_media(|m| {
//!     m.image_url("img.jpg")?;
//!     m.video_url("vid.mp4")  // Decoder will only see this
//! })?;
//! ```
