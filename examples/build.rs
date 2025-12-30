use std::path::PathBuf;

fn main() {
    // Handle code generation for pure Protocol Buffers message types via `piecemeal-build`.
    let out_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("protos")
        .join("piecemeal");

    let config = piecemeal_build::ConfigBuilder::new(
        &["./protos/metrics.proto"],
        out_directory,
        &["./protos"],
    )
    .expect("failed to build `piecemeal-build` configuration");

    piecemeal_build::types::FileDescriptor::run(&config.build())
        .expect("failed to generate pure Protocol Buffers message types");
}
