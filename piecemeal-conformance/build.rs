use piecemeal_build::ConfigBuilder;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let piecemeal_out = out_dir.join("protos").join("piecemeal");
    let prost_out = out_dir.join("protos").join("prost");

    // Discover all `.proto` files under `protos/`.
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
        "Found {} proto files for conformance testing",
        proto_files.len()
    );

    // Generate the builder code for all discovered `.proto` files.
    let proto_paths: Vec<&str> = proto_files
        .iter()
        .map(|p| p.to_str().expect("invalid path"))
        .collect();

    ConfigBuilder::new(&proto_paths[..], &piecemeal_out, &["./protos"])
        .unwrap_or_else(|e| {
            panic!("Failed to create piecemeal-build config: {}", e);
        })
        .compile()
        .unwrap_or_else(|e| {
            panic!("Failed to compile piecemeal code: {}", e);
        });

    println!("Successfully generated piecemeal code for all proto files");

    // Generate owned structs for the discovered `.proto` files via `prost`.
    //
    // We use these for doing roundtripping during conformance tests.

    // Separate the import test files from others since they require special include path handling.
    let (import_protos, other_protos): (Vec<_>, Vec<_>) = proto_files
        .iter()
        .partition(|p| p.starts_with("./protos/imports"));

    std::fs::create_dir_all(&prost_out).unwrap();

    // Compile non-import protos with root include path
    if !other_protos.is_empty() {
        prost_build::Config::new()
            .out_dir(&prost_out)
            .compile_protos(&other_protos, &["./protos"])
            .unwrap_or_else(|e| {
                panic!("Failed to generate prost code for main protos: {}", e);
            });
    }

    // Compile import protos with their subdirectory as include path.
    //
    // This allows cases like "import base_types.proto" to work correctly.
    if !import_protos.is_empty() {
        prost_build::Config::new()
            .out_dir(&prost_out)
            .compile_protos(&import_protos, &["./protos/imports"])
            .unwrap_or_else(|e| {
                panic!("Failed to generate prost code for import protos: {}", e);
            });
    }

    println!("Successfully generated prost code for all proto files");
}
