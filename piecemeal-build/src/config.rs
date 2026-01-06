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
