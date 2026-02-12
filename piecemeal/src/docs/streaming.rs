//! Incremental and streaming serialization patterns.
//!
//! This is the core value proposition of piecemeal: building Protocol Buffers messages
//! incrementally without allocating the entire structure in memory.
//!
//! # The Problem with Traditional Protobuf Libraries
//!
//! Most Protocol Buffers libraries generate struct-based message types:
//!
//! ```ignore
//! // Traditional approach (not piecemeal)
//! struct MetricPayload {
//!     series: Vec<MetricSeries>,
//! }
//!
//! struct MetricSeries {
//!     name: String,
//!     points: Vec<MetricPoint>,
//! }
//! ```
//!
//! This approach has several limitations:
//!
//! 1. **Memory overhead**: You must build the entire message in memory before serializing
//! 2. **Allocation cost**: Nested `Vec`s and `String`s require heap allocations
//! 3. **Borrowing limitations**: You can't easily use `&str` for deeply nested messages
//! 4. **No streaming**: You can't write partial messages to an output stream
//!
//! # How Piecemeal Solves This
//!
//! Piecemeal generates builders that write directly to the output buffer:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! // The scratch buffer handles length-prefix calculation
//! let mut scratch = ScratchWriter::new(Vec::with_capacity(1024));
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! // Each field is written immediately, not stored
//! builder.title("My Post")?;
//! builder.content("This is the content...")?;
//!
//! // Even nested messages are written incrementally
//! builder.author(|author| {
//!     author.name("Jane")?;  // Written now
//!     author.email("jane@example.com")?;  // Written now
//!     Ok(())
//! })?;
//!
//! // Final output
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Memory Efficiency
//!
//! With piecemeal, you can serialize messages using only:
//!
//! - A scratch buffer (reusable across messages)
//! - The final output buffer
//! - Your source data (which can be borrowed)
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! // Source data - could come from anywhere
//! struct BlogPost<'a> {
//!     title: &'a str,
//!     content: &'a str,
//!     tags: &'a [&'a str],
//! }
//!
//! fn serialize_post(post: &BlogPost, output: &mut Vec<u8>) -> std::io::Result<()> {
//!     // Reuse this scratch buffer across multiple serializations
//!     let mut scratch = ScratchWriter::new(Vec::with_capacity(256));
//!     let mut builder = PostBuilder::new(&mut scratch);
//!
//!     // All data is borrowed - no allocations for the content itself
//!     builder.title(post.title)?;
//!     builder.content(post.content)?;
//!     builder.tags(|tags| tags.add_many(post.tags.iter().copied()))?;
//!
//!     builder.finish(output)
//! }
//!
//! let post = BlogPost {
//!     title: "Hello",
//!     content: "World",
//!     tags: &["rust", "protobuf"],
//! };
//!
//! let mut output = Vec::new();
//! serialize_post(&post, &mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Why a Scratch Buffer?
//!
//! Protocol Buffers uses length-delimited encoding for embedded messages. This means
//! the length of a message must be written *before* its contents:
//!
//! ```text
//! [length varint][message bytes...]
//! ```
//!
//! But we don't know the length until we've written all the fields! The scratch buffer
//! solves this by:
//!
//! 1. Writing the message fields to the scratch buffer
//! 2. Calculating the length from what was written
//! 3. Writing the length to the output, then copying the message
//!
//! This is why nested message fields use closures - the closure boundary marks where
//! the nested message ends, triggering the length calculation and copy.
//!
//! # Streaming to Any Output
//!
//! The `finish()` method writes to anything implementing the [`Writer`] trait.
//! The standard library's `Vec<u8>` implements this, but you could implement it for
//! network sockets, files, or other destinations:
//!
//! ```ignore
//! // Write directly to a file
//! builder.finish(&mut file_writer)?;
//!
//! // Write to a network buffer
//! builder.finish(&mut socket_buffer)?;
//! ```
//!
//! # Processing Large Collections
//!
//! For large repeated fields, piecemeal shines because each element is written
//! immediately rather than stored:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! // Imagine this comes from a database cursor or file
//! fn get_tags() -> impl Iterator<Item = &'static str> {
//!     ["tag1", "tag2", "tag3"].into_iter()
//! }
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.title("Post with many tags")?;
//!
//! // Each tag is written as it's produced - no Vec<String> needed
//! builder.tags(|tags| {
//!     for tag in get_tags() {
//!         tags.add(tag)?;
//!     }
//!     Ok(())
//! })?;
//!
//! let mut output = Vec::new();
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Reusing the Scratch Buffer
//!
//! The scratch buffer is cleared after each `finish()` call, so you can reuse it:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::PersonBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::with_capacity(256));
//! let mut output = Vec::new();
//!
//! // First message
//! let mut builder = PersonBuilder::new(&mut scratch);
//! builder.name("Alice")?.age(30)?;
//! builder.finish(&mut output)?;
//!
//! // Second message - scratch buffer is automatically cleared
//! let mut builder = PersonBuilder::new(&mut scratch);
//! builder.name("Bob")?.age(25)?;
//! builder.finish(&mut output)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! [`Writer`]: crate::Writer
