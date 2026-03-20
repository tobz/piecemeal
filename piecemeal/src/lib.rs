//! A library for generating Protocol Buffers messages in an incremental fashion.

#![deny(missing_docs)]

#[cfg(docsrs)]
pub mod docs;

pub mod builder;
pub mod helpers;
pub mod io;
pub mod types;

pub use self::builder::{GenericMapBuilder, MessageMapBuilder, RepeatedBuilder};
pub use self::io::{
    iter::{FieldIter, MapIter, PackedFieldIter},
    reader::{DecodeError, FieldSlice, RawField, Reader},
    scratch::{ScratchBuffer, ScratchWriter},
    writer::Writer,
};
