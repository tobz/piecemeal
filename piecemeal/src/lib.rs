//! A library for generating Protocol Buffers messages in an incremental fashion.

#![deny(missing_docs)]

pub mod builder;
pub mod helpers;
pub mod io;
pub mod types;

pub use self::builder::{GenericMapBuilder, MessageMapBuilder, RepeatedBuilder};
pub use self::io::{
    scratch::{ScratchBuffer, ScratchWriter},
    writer::Writer,
};
