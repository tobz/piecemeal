//! A library for generating Protocol Buffers messages in an incremental fashion.

#![deny(missing_docs)]

pub mod builder;
pub mod helpers;
pub mod io;
pub mod types;

pub use crate::builder::{GenericMapBuilder, MessageMapBuilder, RepeatedBuilder};
pub use crate::io::{
    scratch::{ScratchBuffer, ScratchWriter},
    writer::Writer,
};
