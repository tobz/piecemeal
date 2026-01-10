//! Test that `cargo_output_dir` rejects absolute paths.

use piecemeal_build::ConfigBuilder;

#[test]
fn main() {
    // SAFETY: This runs in its own process, so modifying env vars is safe.
    unsafe {
        std::env::set_var("OUT_DIR", "/tmp");
    }

    let result = ConfigBuilder::new().cargo_output_dir("/absolute/path");
    assert!(
        matches!(
            result,
            Err(piecemeal_build::Error::AbsolutePathNotAllowed(_))
        ),
        "Expected AbsolutePathNotAllowed error, got: {:?}",
        result
    );

    println!("PASSED: cargo_output_dir correctly rejects absolute paths");
}
