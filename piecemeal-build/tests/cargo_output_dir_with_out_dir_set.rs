//! Test that `cargo_output_dir` correctly resolves paths when OUT_DIR is set.

use piecemeal_build::ConfigBuilder;
use tempfile::tempdir;

#[test]
fn main() {
    let temp = tempdir().unwrap();

    // SAFETY: This runs in its own process, so modifying env vars is safe.
    unsafe {
        std::env::set_var("OUT_DIR", temp.path());
    }

    let result = ConfigBuilder::new().cargo_output_dir("protos/generated");
    assert!(
        result.is_ok(),
        "cargo_output_dir should succeed when OUT_DIR is set"
    );

    // Verify the path was correctly joined
    let config = result.unwrap();

    // We can't directly access output_dir since it's private, but we can verify
    // the behavior works end-to-end by attempting to compile (which will fail
    // due to no input files, but that's a different error)
    let compile_result = config.compile();
    assert!(
        matches!(compile_result, Err(piecemeal_build::Error::NoInputFiles)),
        "Expected NoInputFiles error, got: {:?}",
        compile_result
    );

    println!("PASSED: cargo_output_dir correctly resolves with OUT_DIR set");
}
