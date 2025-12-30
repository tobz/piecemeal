//! Conformance tests for piecemeal code generation.
//!
//! This crate serves as an integration test for piecemeal. If this crate
//! compiles successfully, it means:
//! 1. All test proto files were parsed correctly
//! 2. Code generation succeeded for all protos
//! 3. The generated Rust code compiles correctly
//!
//! Runtime tests in the `tests` module further verify that the generated
//! builders work correctly.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/piecemeal/mod.rs"));
}

#[cfg(test)]
mod tests;
