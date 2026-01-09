//! Test that `cargo_output_dir` returns an error when OUT_DIR is not set.

use piecemeal_build::ConfigBuilder;

fn main() {
    // SAFETY: This runs in its own process, so modifying env vars is safe.
    unsafe {
        std::env::remove_var("OUT_DIR");
    }

    let result = ConfigBuilder::new().cargo_output_dir("protos/generated");
    assert!(
        matches!(result, Err(piecemeal_build::Error::OutDirNotSet)),
        "Expected OutDirNotSet error, got: {:?}",
        result
    );

    println!("PASSED: cargo_output_dir correctly errors when OUT_DIR is not set");
}
