//! Test that `output_dir` and `cargo_output_dir` overwrite each other (last-write-wins).

use piecemeal_build::ConfigBuilder;
use std::fs;
use tempfile::tempdir;

fn main() {
    let temp = tempdir().unwrap();

    // SAFETY: This runs in its own process, so modifying env vars is safe.
    unsafe {
        std::env::set_var("OUT_DIR", temp.path());
    }

    // Create a proto file for compilation
    let proto_path = temp.path().join("test.proto");
    fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

    // Test 1: cargo_output_dir followed by output_dir - output_dir wins
    let custom_dir = temp.path().join("custom");
    let result = ConfigBuilder::new()
        .input_files(&[&proto_path])
        .cargo_output_dir("cargo_generated")
        .unwrap()
        .output_dir(&custom_dir)
        .include_paths(&[temp.path()])
        .compile();

    assert!(result.is_ok(), "Compilation should succeed");
    assert!(
        custom_dir.exists(),
        "custom_dir should exist (output_dir wins)"
    );
    assert!(
        !temp.path().join("cargo_generated").exists(),
        "cargo_generated should NOT exist"
    );

    // Clean up for next test
    fs::remove_dir_all(&custom_dir).unwrap();

    // Test 2: output_dir followed by cargo_output_dir - cargo_output_dir wins
    let cargo_dir = temp.path().join("cargo_generated2");
    let result = ConfigBuilder::new()
        .input_files(&[&proto_path])
        .output_dir(&custom_dir)
        .cargo_output_dir("cargo_generated2")
        .unwrap()
        .include_paths(&[temp.path()])
        .compile();

    assert!(result.is_ok(), "Compilation should succeed");
    assert!(
        cargo_dir.exists(),
        "cargo_generated2 should exist (cargo_output_dir wins)"
    );
    assert!(
        !custom_dir.exists(),
        "custom_dir should NOT exist in second test"
    );

    println!("PASSED: output_dir and cargo_output_dir correctly overwrite each other");
}
