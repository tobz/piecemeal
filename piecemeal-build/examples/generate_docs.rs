//! Generate documentation example code.
//!
//! Run with: cargo run --package piecemeal-build --example generate_docs

fn main() {
    let output_dir = std::path::Path::new("piecemeal/src/docs/generated");

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).expect("failed to create output directory");

    piecemeal_build::ConfigBuilder::new()
        .input_files(&[
            "piecemeal/protos/docs/tutorial.proto",
            "piecemeal/protos/docs/blog.proto",
        ])
        .output_dir(output_dir)
        .include_paths(&["piecemeal/protos/docs"])
        .crate_path("crate")
        .compile()
        .expect("failed to compile proto files");

    println!(
        "Generated documentation examples to {}",
        output_dir.display()
    );
}
