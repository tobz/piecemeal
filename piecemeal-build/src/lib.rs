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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_builder_no_proto() {
        let dir = tempdir().unwrap();
        let result = ConfigBuilder::new::<&str, _, &str>(&[], dir.path(), &[]);
        assert!(matches!(result, Err(Error::NoProto)));
    }

    #[test]
    fn test_config_builder_missing_input() {
        let dir = tempdir().unwrap();
        let result = ConfigBuilder::new(&["nonexistent.proto"], dir.path(), &[] as &[&str]);
        assert!(matches!(result, Err(Error::InputFile(_))));
    }

    #[test]
    fn test_config_builder_creates_output_dir() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let output_dir = dir.path().join("output");
        assert!(!output_dir.exists());

        let result = ConfigBuilder::new(&[&proto_path], &output_dir, &[] as &[&str]);

        assert!(result.is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn test_config_builder_existing_output_dir() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        // Output dir already exists
        let result = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_builder_default_values() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str]).unwrap();
        let configs = builder.build();

        assert_eq!(configs.len(), 1);
        assert!(!configs[0].single_module); // default false
        assert!(configs[0].headers); // default true
        assert!(!configs[0].add_deprecated_fields); // default false
    }

    #[test]
    fn test_config_builder_single_module() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str])
            .unwrap()
            .single_module(true);
        let configs = builder.build();

        assert!(configs[0].single_module);
    }

    #[test]
    fn test_config_builder_headers() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str])
            .unwrap()
            .headers(false);
        let configs = builder.build();

        assert!(!configs[0].headers);
    }

    #[test]
    fn test_config_builder_add_deprecated_fields() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str])
            .unwrap()
            .add_deprecated_fields(true);
        let configs = builder.build();

        assert!(configs[0].add_deprecated_fields);
    }

    #[test]
    fn test_config_builder_chained_options() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str])
            .unwrap()
            .single_module(true)
            .headers(false)
            .add_deprecated_fields(true);
        let configs = builder.build();

        assert!(configs[0].single_module);
        assert!(!configs[0].headers);
        assert!(configs[0].add_deprecated_fields);
    }

    #[test]
    fn test_config_builder_default_include_path() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[] as &[&str]).unwrap();
        let configs = builder.build();

        // Should have "." as default include path
        assert!(configs[0].import_search_path.contains(&PathBuf::from(".")));
    }

    #[test]
    fn test_config_builder_custom_include_path() {
        let dir = tempdir().unwrap();
        let proto_path = dir.path().join("test.proto");
        fs::write(&proto_path, "syntax = \"proto3\";").unwrap();

        let include_path = dir.path().join("includes");
        fs::create_dir(&include_path).unwrap();

        let builder = ConfigBuilder::new(&[&proto_path], dir.path(), &[&include_path]).unwrap();
        let configs = builder.build();

        assert!(configs[0].import_search_path.contains(&include_path));
        // Should still have "." appended
        assert!(configs[0].import_search_path.contains(&PathBuf::from(".")));
    }

    #[test]
    fn test_config_builder_multiple_input_files() {
        let dir = tempdir().unwrap();
        let proto1 = dir.path().join("test1.proto");
        let proto2 = dir.path().join("test2.proto");
        fs::write(&proto1, "syntax = \"proto3\";").unwrap();
        fs::write(&proto2, "syntax = \"proto3\";").unwrap();

        let builder = ConfigBuilder::new(&[&proto1, &proto2], dir.path(), &[] as &[&str]).unwrap();
        let configs = builder.build();

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].in_file, proto1);
        assert_eq!(configs[1].in_file, proto2);
    }
}
