//! Conformance tests for `piecemeal` code generation.
//!
//! This crate serves as an integration test. If this crate compiles successfully, it means:
//!
//! 1. All test proto files were parsed correctly.
//! 2. Code generation succeeded for all protos.
//! 3. The generated Rust code compiles correctly.
//!
//! Runtime tests in the `tests` module further verify that the generated builders work correctly, and roundtrip tests
//! decode with `prost` to verify the output is valid Protocol Buffers.

#![allow(dead_code)]
#![allow(unused_imports)]

/// `piecemeal`-generated builders for encoding protobuf messages.
pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/piecemeal/mod.rs"));
}

/// `prost`-generated structs for decoding protobuf messages (used in roundtrip tests).
#[cfg(test)]
pub mod prost_protos {
    pub mod scalars {
        pub mod all_scalar_types {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/scalars.all_scalar_types.rs"
            ));
        }
    }
    pub mod enums {
        pub mod basic_enum {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/enums.basic_enum.rs"
            ));
        }
    }
    pub mod messages {
        pub mod nested_messages {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/messages.nested_messages.rs"
            ));
        }
        pub mod empty_message {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/messages.empty_message.rs"
            ));
        }
    }
    pub mod oneofs {
        pub mod basic_oneof {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/oneofs.basic_oneof.rs"
            ));
        }
    }
    pub mod repeated {
        pub mod repeated_scalars {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/repeated.repeated_scalars.rs"
            ));
        }
    }
    pub mod maps {
        pub mod map_scalar_scalar {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/maps.map_scalar_scalar.rs"
            ));
        }
        pub mod map_scalar_message {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/maps.map_scalar_message.rs"
            ));
        }
    }
    pub mod imports {
        pub mod base_types {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/imports.base_types.rs"
            ));
        }
        pub mod importing_file {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/imports.importing_file.rs"
            ));
        }
    }
    pub mod edge_cases {
        pub mod reserved_keywords {
            include!(concat!(
                env!("OUT_DIR"),
                "/protos/prost/edge_cases.reserved_keywords.rs"
            ));
        }
    }
}

#[cfg(test)]
mod tests;
