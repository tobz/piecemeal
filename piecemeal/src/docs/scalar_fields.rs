//! Scalar fields and type mappings.
//!
//! Piecemeal supports all Protocol Buffers scalar types, mapping them to appropriate
//! Rust types in the generated builder methods.
//!
//! # Supported Scalar Types
//!
//! | Proto Type   | Rust Type | Wire Format |
//! |--------------|-----------|-------------|
//! | `int32`      | `i32`     | Varint |
//! | `int64`      | `i64`     | Varint |
//! | `uint32`     | `u32`     | Varint |
//! | `uint64`     | `u64`     | Varint |
//! | `sint32`     | `i32`     | Varint (ZigZag) |
//! | `sint64`     | `i64`     | Varint (ZigZag) |
//! | `bool`       | `bool`    | Varint |
//! | `fixed32`    | `u32`     | Fixed 4 bytes |
//! | `fixed64`    | `u64`     | Fixed 8 bytes |
//! | `sfixed32`   | `i32`     | Fixed 4 bytes |
//! | `sfixed64`   | `i64`     | Fixed 8 bytes |
//! | `float`      | `f32`     | Fixed 4 bytes |
//! | `double`     | `f64`     | Fixed 8 bytes |
//! | `string`     | `&str`    | Length-delimited |
//! | `bytes`      | `&[u8]`   | Length-delimited |
//!
//! # Using Scalar Fields
//!
//! Each scalar field generates a method that takes the appropriate Rust type:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::PersonBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PersonBuilder::new(&mut scratch);
//!
//! // int32 field takes i32
//! builder.age(30)?;
//!
//! // string field takes &str
//! builder.name("Alice")?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Integer Fields
//!
//! Integer fields use the standard Rust integer types. Use `as` or `.into()` for
//! conversions when needed:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! // int64 field takes i64
//! let count: u32 = 1000;
//! builder.view_count(count as i64)?;
//!
//! // Or using .into() for lossless conversions
//! let small_count: i32 = 500;
//! builder.view_count(small_count.into())?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Floating-Point Fields
//!
//! Floating-point fields use `f32` for `float` and `f64` for `double`:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! // double field takes f64
//! builder.rating(4.5)?;
//!
//! // Convert from f32 if needed
//! let rating: f32 = 4.5;
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PostBuilder::new(&mut scratch);
//! builder.rating(rating.into())?;
//!
//! // Convert from integers
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PostBuilder::new(&mut scratch);
//! builder.rating(5.0)?;  // Use floating-point literal
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # String and Bytes Fields
//!
//! String fields accept `&str` (borrowed strings), which means you don't need to
//! allocate a `String`:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::tutorial::PersonBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PersonBuilder::new(&mut scratch);
//!
//! // String literals work directly
//! builder.name("Alice")?;
//!
//! // So do borrowed strings
//! let name = String::from("Bob");
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PersonBuilder::new(&mut scratch);
//! builder.name(&name)?;
//!
//! // And string slices
//! let full_name = "Charlie Brown";
//! # let mut scratch = ScratchWriter::new(Vec::new());
//! # let mut builder = PersonBuilder::new(&mut scratch);
//! builder.name(&full_name[..7])?;  // "Charlie"
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Similarly, `bytes` fields accept `&[u8]`:
//!
//! ```ignore
//! // For a `bytes data = 1;` field:
//! builder.data(&[0x01, 0x02, 0x03])?;
//! builder.data(my_vec.as_slice())?;
//! ```
//!
//! # Boolean Fields
//!
//! Boolean fields take Rust `bool` values:
//!
//! ```
//! use piecemeal::ScratchWriter;
//! use piecemeal::docs::blog::PostBuilder;
//!
//! let mut scratch = ScratchWriter::new(Vec::new());
//! let mut builder = PostBuilder::new(&mut scratch);
//!
//! builder.is_featured(true)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Why Borrowed Types?
//!
//! Piecemeal uses borrowed types like `&str` instead of owned types like `String`
//! because it writes data directly to the output buffer. This means:
//!
//! - No extra allocations for string data
//! - You can serialize data from any source without copying
//! - The borrowed data only needs to live until the field method returns
