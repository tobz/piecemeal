pub mod errors;
mod keywords;
mod parser;
pub mod types;

use errors::{Error, Result};
use std::path::{Path, PathBuf};
use types::Config;

/// Builder for generating the configuration for Protocol Buffers code generation.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    in_files: Vec<PathBuf>,
    output_dir: PathBuf,
    include_paths: Vec<PathBuf>,
    single_module: bool,
    headers: bool,
    add_deprecated_fields: bool,
}

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder from the given input files, include directories, and output directory.
    ///
    /// # Errors
    ///
    /// If no input files are provided, if they don't exist, or if the output directory doesn't exist and can't be
    /// created, an error is returned.
    pub fn new<I, O, IP>(
        in_files: &[I],
        output_dir: O,
        include_paths: &[IP],
    ) -> Result<ConfigBuilder>
    where
        I: AsRef<Path>,
        O: AsRef<Path>,
        IP: AsRef<Path>,
    {
        let in_files = in_files
            .iter()
            .map(|f| f.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        if in_files.is_empty() {
            return Err(Error::NoProto);
        }

        for f in &in_files {
            if !f.exists() {
                return Err(Error::InputFile(format!("{}", f.display())));
            }
        }

        let output_dir = output_dir.as_ref().to_path_buf();
        if !output_dir.is_dir() {
            std::fs::create_dir_all(&output_dir).map_err(Error::FailedToCreateOutputDirectory)?;
        }

        let mut include_paths = include_paths
            .iter()
            .map(|f| f.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        let default = PathBuf::from(".");
        if include_paths.is_empty() || !include_paths.contains(&default) {
            include_paths.push(default);
        }

        Ok(ConfigBuilder {
            in_files,
            output_dir,
            include_paths,
            headers: true,
            ..Default::default()
        })
    }

    /// Omit the generation of modules for each package when there is only one package.
    ///
    /// Defaults to `false`.
    pub fn single_module(mut self, val: bool) -> Self {
        self.single_module = val;
        self
    }

    /// Whether or not to emit certain attribute headers in the generated code to suppress various Clippy lints, such as
    /// missing documentation or dead code, and so on.
    ///
    /// Defaults to `true`.
    pub fn headers(mut self, val: bool) -> Self {
        self.headers = val;
        self
    }

    /// Whether or not to add deprecated fields to the generated code or not.
    ///
    /// If set to `true`, deprecated fields will be added to the generated code and all relevant methods for interacting
    /// with them will be annotated with `#[deprecated]`.
    ///
    /// Defaults to `false`.
    pub fn add_deprecated_fields(mut self, val: bool) -> Self {
        self.add_deprecated_fields = val;
        self
    }

    /// Builds a list of code generation configurations, one for each input file.
    pub fn build(self) -> Vec<Config> {
        self.in_files
            .into_iter()
            .map(|in_file| {
                let out_dir = self.output_dir.clone();

                Config {
                    in_file,
                    out_dir,
                    import_search_path: self.include_paths.clone(),
                    single_module: self.single_module,
                    headers: self.headers,
                    add_deprecated_fields: self.add_deprecated_fields,
                }
            })
            .collect()
    }
}
