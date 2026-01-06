use std::path::PathBuf;

use piecemeal_build::ConfigBuilder;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos/metrics.proto");

    // Handle code generation for pure Protocol Buffers message types via `piecemeal-build`.
    let out_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("protos")
        .join("piecemeal");

    ConfigBuilder::new(&["./protos/metrics.proto"], out_directory, &["./protos"])
        .expect("failed to build `piecemeal-build` configuration")
        .compile()
        .expect("failed to generate pure Protocol Buffers message types");
}
