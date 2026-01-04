use std::io;

/// Main error type when building.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(io::Error),

    /// Failed to parse the Protocol Buffers definition.
    Nom(nom::Err<nom::error::Error<String>>),

    /// Additional data remaining in the .proto file after parsing.
    TrailingGarbage(String),

    /// No .proto file was provided.
    NoProto,

    /// Failed to read input file.
    InputFile(String),

    /// Failed to read output file.
    OutputFile(String),

    /// Failed to create output directory.
    FailedToCreateOutputDirectory(io::Error),

    /// Multiple input files with `--output` argument
    OutputMultipleInputs,

    /// Encountered an invalid message definition.
    InvalidMessage(String),

    /// Varint decoding error
    InvalidImport(String),

    /// Empty read
    EmptyRead,

    /// Enum or message not found
    MessageOrEnumNotFound(String),

    /// Invalid default enum
    InvalidDefaultEnum(String),

    /// Detected a cycle in the definition.
    Cycle(Vec<String>),
}

/// A wrapper for `Result<T, Error>`
pub type Result<T> = ::std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Nom(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Nom(e) => write!(f, "{}", e),
            Error::TrailingGarbage(s) => {
                write!(f, "additional data in .proto file after parsing: {:?}", s)
            }
            Error::NoProto => write!(f, "no .proto file was provided"),
            Error::InputFile(file) => write!(f, "failed to read input file '{}'", file),
            Error::OutputFile(file) => write!(f, "failed to read output file '{}'", file),
            Error::FailedToCreateOutputDirectory(e) => {
                write!(f, "failed to create output directory: {}", e)
            }
            Error::OutputMultipleInputs => write!(f, "--output only allowed for single input file"),
            Error::InvalidMessage(msg) => write!(
                f,
                "Message checks errored: {}\r\n\
                Proto definition might be invalid or something got wrong in the parsing",
                msg
            ),
            Error::InvalidImport(imp) => write!(
                f,
                "could not convert Protocol Buffers import into module import: {} (import definition might be invalid, or some characters may not be supported)",
                imp
            ),
            Error::EmptyRead => write!(
                f,
                "no messages or enums were read (definition may be invalid or only unsupported structures were defined)"
            ),
            Error::MessageOrEnumNotFound(me) => {
                write!(f, "could not find message or enum '{}'", me)
            }
            Error::InvalidDefaultEnum(en) => {
                write!(
                    f,
                    "enum field cannot be set to '{}': variant does not exist",
                    en
                )
            }
            Error::Cycle(msgs) => write!(
                f,
                "messages {:?} are cyclic (missing an optional field)",
                msgs
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    // Display tests for all error variants
    #[test]
    fn test_display_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let err = Error::Io(io_err);
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_display_trailing_garbage() {
        let err = Error::TrailingGarbage("extra stuff".to_string());
        let display = err.to_string();
        assert!(display.contains("additional data"));
        assert!(display.contains("extra stuff"));
    }

    #[test]
    fn test_display_no_proto() {
        let err = Error::NoProto;
        assert!(err.to_string().contains("no .proto file"));
    }

    #[test]
    fn test_display_input_file() {
        let err = Error::InputFile("missing.proto".to_string());
        let display = err.to_string();
        assert!(display.contains("failed to read input file"));
        assert!(display.contains("missing.proto"));
    }

    #[test]
    fn test_display_output_file() {
        let err = Error::OutputFile("output.rs".to_string());
        let display = err.to_string();
        assert!(display.contains("failed to read output file"));
        assert!(display.contains("output.rs"));
    }

    #[test]
    fn test_display_failed_to_create_output_directory() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err = Error::FailedToCreateOutputDirectory(io_err);
        let display = err.to_string();
        assert!(display.contains("failed to create output directory"));
    }

    #[test]
    fn test_display_output_multiple_inputs() {
        let err = Error::OutputMultipleInputs;
        assert!(err.to_string().contains("--output"));
    }

    #[test]
    fn test_display_invalid_message() {
        let err = Error::InvalidMessage("bad field".to_string());
        let display = err.to_string();
        assert!(display.contains("Message checks errored"));
        assert!(display.contains("bad field"));
    }

    #[test]
    fn test_display_invalid_import() {
        let err = Error::InvalidImport("unknown.proto".to_string());
        let display = err.to_string();
        assert!(display.contains("could not convert"));
        assert!(display.contains("unknown.proto"));
    }

    #[test]
    fn test_display_empty_read() {
        let err = Error::EmptyRead;
        assert!(err.to_string().contains("no messages or enums"));
    }

    #[test]
    fn test_display_message_or_enum_not_found() {
        let err = Error::MessageOrEnumNotFound("Missing".to_string());
        let display = err.to_string();
        assert!(display.contains("could not find message or enum"));
        assert!(display.contains("Missing"));
    }

    #[test]
    fn test_display_invalid_default_enum() {
        let err = Error::InvalidDefaultEnum("UNKNOWN".to_string());
        let display = err.to_string();
        assert!(display.contains("enum field cannot be set"));
        assert!(display.contains("UNKNOWN"));
    }

    #[test]
    fn test_display_cycle() {
        let err = Error::Cycle(vec!["A".to_string(), "B".to_string()]);
        let display = err.to_string();
        assert!(display.contains("cyclic"));
        assert!(display.contains("A"));
        assert!(display.contains("B"));
    }

    // Error::source() tests
    #[test]
    fn test_source_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err = Error::Io(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_source_none_for_simple_variants() {
        assert!(Error::NoProto.source().is_none());
        assert!(Error::EmptyRead.source().is_none());
        assert!(Error::OutputMultipleInputs.source().is_none());
        assert!(Error::InputFile("test".to_string()).source().is_none());
        assert!(Error::OutputFile("test".to_string()).source().is_none());
        assert!(
            Error::TrailingGarbage("test".to_string())
                .source()
                .is_none()
        );
        assert!(Error::InvalidMessage("test".to_string()).source().is_none());
        assert!(Error::InvalidImport("test".to_string()).source().is_none());
        assert!(
            Error::MessageOrEnumNotFound("test".to_string())
                .source()
                .is_none()
        );
        assert!(
            Error::InvalidDefaultEnum("test".to_string())
                .source()
                .is_none()
        );
        assert!(Error::Cycle(vec!["A".to_string()]).source().is_none());
    }

    #[test]
    fn test_source_failed_to_create_output_directory() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err = Error::FailedToCreateOutputDirectory(io_err);
        // This variant doesn't have source() returning Some
        assert!(err.source().is_none());
    }

    // From trait tests
    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
