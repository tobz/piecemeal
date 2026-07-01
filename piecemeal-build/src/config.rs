use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::{errors::Error, types::FileDescriptor};

/// Configuration builder for Protocol Buffers code generation.
#[derive(Debug)]
pub struct ConfigBuilder {
    input_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    include_paths: Vec<PathBuf>,
    crate_path: String,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            output_dir: None,
            include_paths: Vec::new(),
            crate_path: "::piecemeal".to_string(),
        }
    }
}

impl ConfigBuilder {
    /// Creates a new, empty `ConfigBuilder`.
    ///
    /// Use the builder methods to configure input files, output directory, and include paths,
    /// then call [`compile()`](Self::compile) to generate code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ConfigBuilder::new()
    ///     .input_files(&["./protos/messages.proto"])
    ///     .cargo_output_dir("protos")?
    ///     .include_paths(&["./protos"])
    ///     .compile()?;
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the input `.proto` files to compile.
    ///
    /// This replaces any previously configured input files.
    pub fn input_files<I>(mut self, files: &[I]) -> Self
    where
        I: AsRef<Path>,
    {
        self.input_files = files.iter().map(|f| f.as_ref().to_path_buf()).collect();
        self
    }

    /// Sets the output directory for generated code.
    ///
    /// The path is used as-is. If you want to resolve a relative path against
    /// Cargo's `OUT_DIR`, use [`cargo_output_dir()`](Self::cargo_output_dir) instead.
    ///
    /// This overwrites any previously configured output directory.
    pub fn output_dir<P>(mut self, dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.output_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Sets the output directory relative to Cargo's `OUT_DIR`.
    ///
    /// The provided path must be relative and will be joined with the `OUT_DIR`
    /// environment variable.
    ///
    /// This overwrites any previously configured output directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `OUT_DIR` environment variable is not set
    /// - The provided path is absolute
    pub fn cargo_output_dir<P>(mut self, dir: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let dir = dir.as_ref();

        if dir.is_absolute() {
            return Err(Error::AbsolutePathNotAllowed(
                dir.to_string_lossy().to_string(),
            ));
        }

        let out_dir = std::env::var("OUT_DIR").map_err(|_| Error::OutDirNotSet)?;

        self.output_dir = Some(PathBuf::from(out_dir).join(dir));
        Ok(self)
    }

    /// Sets the include paths for resolving imports in `.proto` files.
    ///
    /// This replaces any previously configured include paths.
    pub fn include_paths<I>(mut self, paths: &[I]) -> Self
    where
        I: AsRef<Path>,
    {
        self.include_paths = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
        self
    }

    /// Sets the path used to reference the piecemeal crate in generated code.
    ///
    /// Defaults to `::piecemeal`. Use `crate` when generating code that will
    /// live inside the piecemeal crate itself.
    pub fn crate_path(mut self, path: &str) -> Self {
        self.crate_path = path.to_string();
        self
    }

    /// Compiles the configured `.proto` files and generates builder code for them.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No input files were configured
    /// - An input file does not exist
    /// - No output directory was configured
    /// - The output directory cannot be created
    /// - There is an error parsing the input files or generating code
    pub fn compile(self) -> Result<(), Error> {
        // Validate input files
        if self.input_files.is_empty() {
            return Err(Error::NoInputFiles);
        }

        for input_file in &self.input_files {
            if !input_file.exists() {
                return Err(Error::InputFileDoesNotExist(
                    input_file.to_string_lossy().to_string(),
                ));
            }
        }

        // Validate and create output directory
        let output_dir = self.output_dir.ok_or(Error::NoOutputDirectory)?;
        if !output_dir.is_dir() {
            std::fs::create_dir_all(&output_dir).map_err(Error::FailedToCreateOutputDirectory)?;
        }

        // Prepare include paths, ensuring current directory is included
        let mut include_paths = self.include_paths;
        let default = PathBuf::from(".");
        if include_paths.is_empty() || !include_paths.contains(&default) {
            include_paths.push(default);
        }

        // Parse all input files without resolving types. Using is_import: true
        // skips resolve_types and sanity_checks so that MessageOrEnum(String)
        // references remain unresolved. This lets us merge descriptors that share
        // a package and then resolve types once against the combined message list.
        let mut grouped: HashMap<String, FileDescriptor> = HashMap::new();
        let mut no_package: Vec<FileDescriptor> = Vec::new();

        for input_file in self.input_files {
            let descriptor =
                FileDescriptor::try_from_input_file_internal(&input_file, &include_paths, true)?;

            if descriptor.package.is_empty() {
                // Empty package: output path is based on input filename,
                // so each file naturally gets its own output. No merging.
                no_package.push(descriptor);
            } else {
                match grouped.entry(descriptor.package.clone()) {
                    Entry::Occupied(mut entry) => {
                        let existing = entry.get_mut();
                        existing.messages.extend(descriptor.messages);
                        existing.enums.extend(descriptor.enums);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(descriptor);
                    }
                }
            }
        }

        // Determine which packages are "parents" — i.e., another package has them
        // as a prefix. Parent packages must be written to <name>/mod.rs instead of
        // <name>.rs so they don't conflict with the sub-package directory.
        let parent_packages: HashSet<String> = grouped
            .keys()
            .filter(|pkg| {
                let prefix = format!("{}.", pkg);
                grouped.keys().any(|other| other.starts_with(&prefix))
            })
            .cloned()
            .collect();

        // Resolve types and write output for each (possibly merged) descriptor.
        let mut descriptors: Vec<_> = grouped.into_values().collect();
        descriptors.sort_by_key(|d| d.package.len());
        for mut descriptor in descriptors.into_iter().chain(no_package) {
            let is_parent = parent_packages.contains(&descriptor.package);
            descriptor.resolve_types()?;
            // Re-apply proto3 defaults (e.g. implicit packing of repeated scalar
            // fields) now that types are resolved. During parsing these files went
            // through with is_import: true, so set_defaults() ran before types were
            // resolved and its nested-message BFS — which keys off resolved message
            // types — never descended into nested messages. Running it here ensures
            // nested messages get the same packing treatment as top-level ones.
            descriptor.set_defaults()?;
            descriptor.sanity_checks()?;
            descriptor.write_to_file(&output_dir, &self.crate_path, is_parent)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compile_no_input_files() {
        let temp = tempdir().unwrap();
        let result = ConfigBuilder::new()
            .output_dir(temp.path())
            .include_paths(&["./"])
            .compile();
        assert!(matches!(result, Err(Error::NoInputFiles)));
    }

    #[test]
    fn test_compile_input_file_does_not_exist() {
        let temp = tempdir().unwrap();
        let result = ConfigBuilder::new()
            .input_files(&["nonexistent.proto"])
            .output_dir(temp.path())
            .include_paths(&["./"])
            .compile();
        assert!(matches!(result, Err(Error::InputFileDoesNotExist(_))));
    }

    #[test]
    fn test_compile_no_output_directory() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        let result = ConfigBuilder::new()
            .input_files(&[&proto_path])
            .include_paths(&[temp.path()])
            .compile();
        assert!(matches!(result, Err(Error::NoOutputDirectory)));
    }

    #[test]
    fn test_compile_creates_output_directory() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        let output_dir = temp.path().join("nested").join("output");
        assert!(!output_dir.exists());

        let result = ConfigBuilder::new()
            .input_files(&[&proto_path])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile();
        assert!(result.is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn test_compile_with_existing_output_directory() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        let output_dir = temp.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = ConfigBuilder::new()
            .input_files(&[&proto_path])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile();
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_success() {
        let temp = tempdir().unwrap();
        let proto_content = r#"syntax = "proto3";
package test;
message TestMessage {
    string name = 1;
    int32 value = 2;
}
"#;
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, proto_content).unwrap();

        let output_dir = temp.path().join("output");
        let result = ConfigBuilder::new()
            .input_files(&[&proto_path])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile();
        assert!(result.is_ok());

        // Verify output file was created
        let output_file = output_dir.join("test.rs");
        assert!(output_file.exists());
    }

    #[test]
    fn test_compile_multiple_files() {
        let temp = tempdir().unwrap();

        let proto1 = temp.path().join("first.proto");
        fs::write(
            &proto1,
            "syntax = \"proto3\";\npackage first;\nmessage First {}\n",
        )
        .unwrap();

        let proto2 = temp.path().join("second.proto");
        fs::write(
            &proto2,
            "syntax = \"proto3\";\npackage second;\nmessage Second {}\n",
        )
        .unwrap();

        let output_dir = temp.path().join("output");
        let result = ConfigBuilder::new()
            .input_files(&[&proto1, &proto2])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile();
        assert!(result.is_ok());

        assert!(output_dir.join("first.rs").exists());
        assert!(output_dir.join("second.rs").exists());
    }

    #[test]
    fn test_compile_multiple_files_same_package() {
        let temp = tempdir().unwrap();

        let proto1 = temp.path().join("messages.proto");
        fs::write(
            &proto1,
            "syntax = \"proto3\";\npackage shared;\nmessage First { string name = 1; }\n",
        )
        .unwrap();

        let proto2 = temp.path().join("more_messages.proto");
        fs::write(
            &proto2,
            "syntax = \"proto3\";\npackage shared;\nmessage Second { int32 value = 1; }\n",
        )
        .unwrap();

        let output_dir = temp.path().join("output");
        ConfigBuilder::new()
            .input_files(&[&proto1, &proto2])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile()
            .unwrap();

        let content = fs::read_to_string(output_dir.join("shared.rs")).unwrap();
        assert!(
            content.contains("pub struct FirstBuilder"),
            "First message missing from output"
        );
        assert!(
            content.contains("pub struct SecondBuilder"),
            "Second message missing from output"
        );
    }

    #[test]
    fn test_compile_nested_message_repeated_scalar_is_packed() {
        // Regression test: proto3 implicitly packs repeated scalar fields. This must
        // apply to nested messages as well as top-level ones. Because ConfigBuilder
        // parses every file with is_import: true, packing must be (re)applied after
        // type resolution or the nested-message BFS never descends into Inner.
        let temp = tempdir().unwrap();

        let proto = temp.path().join("nested.proto");
        fs::write(
            &proto,
            "syntax = \"proto3\";\n\
             message Outer {\n\
             \x20 message Inner { repeated int32 values = 1; }\n\
             \x20 Inner inner = 1;\n\
             }\n",
        )
        .unwrap();

        let output_dir = temp.path().join("output");
        ConfigBuilder::new()
            .input_files(&[&proto])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile()
            .unwrap();

        let content = fs::read_to_string(output_dir.join("nested.rs")).unwrap();
        assert!(
            content.contains("RepeatedBuilder::new(1, true,"),
            "nested repeated scalar field should be packed, got:\n{content}"
        );
    }

    #[test]
    fn test_compile_parent_and_child_packages() {
        let temp = tempdir().unwrap();

        let proto1 = temp.path().join("parent.proto");
        fs::write(
            &proto1,
            "syntax = \"proto3\";\npackage a.b;\nmessage Parent { string name = 1; }\n",
        )
        .unwrap();

        let proto2 = temp.path().join("child.proto");
        fs::write(
            &proto2,
            "syntax = \"proto3\";\npackage a.b.c;\nmessage Child { int32 value = 1; }\n",
        )
        .unwrap();

        let output_dir = temp.path().join("output");
        ConfigBuilder::new()
            .input_files(&[&proto1, &proto2])
            .output_dir(&output_dir)
            .include_paths(&[temp.path()])
            .compile()
            .unwrap();

        // Parent package should be at a/b/mod.rs (not a/b.rs) since it has a child package
        assert!(
            !output_dir.join("a/b.rs").exists(),
            "a/b.rs should not exist when a/b/ directory is needed"
        );
        let parent_content = fs::read_to_string(output_dir.join("a/b/mod.rs")).unwrap();
        assert!(
            parent_content.contains("pub struct ParentBuilder"),
            "Parent types missing from a/b/mod.rs"
        );
        // mod.rs should also declare the child module
        assert!(
            parent_content.contains("pub mod c;"),
            "Child module declaration missing from a/b/mod.rs"
        );

        // Child package should be at a/b/c.rs
        let child_content = fs::read_to_string(output_dir.join("a/b/c.rs")).unwrap();
        assert!(
            child_content.contains("pub struct ChildBuilder"),
            "Child types missing from a/b/c.rs"
        );
    }
}
