use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos");

    let out_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("protos")
        .join("piecemeal");

    // Discover all .proto files under protos/
    let proto_files: Vec<PathBuf> = WalkDir::new("./protos")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "proto"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if proto_files.is_empty() {
        panic!("No .proto files found in protos/ directory");
    }

    println!(
        "cargo:warning=piecemeal-conformance: Found {} proto files for conformance testing",
        proto_files.len()
    );

    // Convert to string slices for the API
    let proto_paths: Vec<&str> = proto_files
        .iter()
        .map(|p| p.to_str().expect("invalid path"))
        .collect();

    let config =
        piecemeal_build::ConfigBuilder::new(&proto_paths[..], &out_directory, &["./protos"])
            .unwrap_or_else(|e| {
                panic!("Failed to create piecemeal-build config: {}", e);
            });

    piecemeal_build::types::FileDescriptor::run(&config.build()).unwrap_or_else(|e| {
        panic!("Failed to generate code: {}", e);
    });

    println!(
        "cargo:warning=piecemeal-conformance: Successfully generated code for all proto files"
    );
}
