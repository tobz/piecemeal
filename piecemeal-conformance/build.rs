use piecemeal_build::ConfigBuilder;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Validates a proto file using protoc.
///
/// Returns Ok(()) if protoc accepts the file, Err with stderr output if rejected.
fn validate_with_protoc(proto_file: &Path, include_paths: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new("protoc");

    for path in include_paths {
        cmd.arg(format!("--proto_path={}", path));
    }

    // Validate only, output descriptor to /dev/null (or NUL on Windows)
    cmd.arg("-o")
        .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
    cmd.arg(proto_file);

    let output = cmd
        .output()
        .expect("protoc must be installed and available in PATH");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Discovers all .proto files in a directory.
fn discover_protos(dir: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "proto"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Gets the appropriate include paths for a proto file.
fn get_include_paths(proto_path: &Path) -> Vec<&'static str> {
    if proto_path.starts_with("./protos/imports") {
        vec!["./protos/imports"]
    } else {
        vec!["./protos"]
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let piecemeal_out = out_dir.join("protos").join("piecemeal");
    let prost_out = out_dir.join("protos").join("prost");

    // Discover valid `.proto` files under `protos/` (excluding `protos/invalid/`).
    let valid_protos: Vec<PathBuf> = discover_protos("./protos")
        .into_iter()
        .filter(|p| !p.starts_with("./protos/invalid"))
        .collect();

    if valid_protos.is_empty() {
        panic!("No .proto files found in protos/ directory");
    }

    println!(
        "Found {} valid proto files for conformance testing",
        valid_protos.len()
    );

    // Validate all valid protos with protoc to ensure they're actually valid
    println!("Validating protos with protoc...");
    for proto_path in &valid_protos {
        let include_paths = get_include_paths(proto_path);
        if let Err(err) = validate_with_protoc(proto_path, &include_paths) {
            panic!("protoc rejected proto {}: {}", proto_path.display(), err);
        }
    }
    println!("All protos passed protoc validation");

    // Generate the builder code for all valid `.proto` files.
    let proto_paths: Vec<&str> = valid_protos
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
    let (import_protos, other_protos): (Vec<_>, Vec<_>) = valid_protos
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
