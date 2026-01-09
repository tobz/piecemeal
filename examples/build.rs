use piecemeal_build::ConfigBuilder;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos/metrics.proto");

    // Handle code generation for pure Protocol Buffers message types via `piecemeal-build`.
    ConfigBuilder::new()
        .input_files(&["./protos/metrics.proto"])
        .cargo_output_dir("protos/piecemeal")
        .expect("failed to resolve cargo output directory")
        .include_paths(&["./protos"])
        .compile()
        .expect("failed to generate pure Protocol Buffers message types");
}
