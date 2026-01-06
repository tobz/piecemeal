use std::path::{Path, PathBuf};

use crate::{errors::Error, types::FileDescriptor};

/// Configuration build for Protocol Buffers code generation.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    input_files: Vec<PathBuf>,
    output_dir: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder from the given input files, include directories, and output directory.
    ///
    /// # Errors
    ///
    /// If no input files are provided, if they don't exist, or if the output directory doesn't exist and can't be
    /// created, an error is returned.
    pub fn new<I, O, IP>(
        input_files: &[I],
        output_dir: O,
        include_paths: &[IP],
    ) -> Result<Self, Error>
    where
        I: AsRef<Path>,
        O: AsRef<Path>,
        IP: AsRef<Path>,
    {
        // Get our input files, and make sure they all exist on disk.
        let input_files = input_files
            .iter()
            .map(|f| f.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        if input_files.is_empty() {
            return Err(Error::NoInputFiles);
        }

        for input_file in &input_files {
            if !input_file.exists() {
                return Err(Error::InputFileDoesNotExist(
                    input_file.to_string_lossy().to_string(),
                ));
            }
        }

        // Make sure our output directory exists on disk, creating it recursively if not.
        let output_dir = output_dir.as_ref().to_path_buf();
        if !output_dir.is_dir() {
            std::fs::create_dir_all(&output_dir).map_err(Error::FailedToCreateOutputDirectory)?;
        }

        // Get all of the import paths, making sure we always include the current directory among them.
        let mut include_paths = include_paths
            .iter()
            .map(|f| f.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        let default = PathBuf::from(".");
        if include_paths.is_empty() || !include_paths.contains(&default) {
            include_paths.push(default);
        }

        Ok(Self {
            input_files,
            output_dir,
            include_paths,
        })
    }

    /// Compiles the configured `.proto` files and generates builder code for them.
    ///
    /// # Errors
    ///
    /// If there an error reading the input files, parsing them, or generating the builder code, an error is returned.
    pub fn compile(self) -> Result<(), Error> {
        for input_file in self.input_files {
            let descriptor = FileDescriptor::try_from_input_file(&input_file, &self.include_paths)?;
            descriptor.write_to_file(&self.output_dir)?;
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
    fn test_new_no_input_files() {
        let temp = tempdir().unwrap();
        let result = ConfigBuilder::new::<&str, _, _>(&[], temp.path(), &["./"]);
        assert!(matches!(result, Err(Error::NoInputFiles)));
    }

    #[test]
    fn test_new_input_file_does_not_exist() {
        let temp = tempdir().unwrap();
        let result = ConfigBuilder::new(&["nonexistent.proto"], temp.path(), &["./"]);
        assert!(matches!(result, Err(Error::InputFileDoesNotExist(_))));
    }

    #[test]
    fn test_new_creates_output_directory() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        let output_dir = temp.path().join("nested").join("output");
        assert!(!output_dir.exists());

        let result = ConfigBuilder::new(&[&proto_path], &output_dir, &[temp.path()]);
        assert!(result.is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn test_new_with_existing_output_directory() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        let output_dir = temp.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = ConfigBuilder::new(&[&proto_path], &output_dir, &[temp.path()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_adds_default_include_path() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        // Test with empty include paths - should add "."
        let result = ConfigBuilder::new::<_, _, &str>(&[&proto_path], temp.path(), &[]);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.include_paths.contains(&PathBuf::from(".")));
    }

    #[test]
    fn test_new_does_not_duplicate_default_include_path() {
        let temp = tempdir().unwrap();
        let proto_path = temp.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";\nmessage Empty {}\n").unwrap();

        // Test with "." already in include paths - should not duplicate
        let result = ConfigBuilder::new(&[&proto_path], temp.path(), &["."]);
        assert!(result.is_ok());
        let config = result.unwrap();
        let dot_count = config
            .include_paths
            .iter()
            .filter(|p| *p == &PathBuf::from("."))
            .count();
        assert_eq!(dot_count, 1);
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
        let config = ConfigBuilder::new(&[&proto_path], &output_dir, &[temp.path()]).unwrap();
        let result = config.compile();
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
        let config = ConfigBuilder::new(&[&proto1, &proto2], &output_dir, &[temp.path()]).unwrap();
        let result = config.compile();
        assert!(result.is_ok());

        assert!(output_dir.join("first.rs").exists());
        assert!(output_dir.join("second.rs").exists());
    }
}
