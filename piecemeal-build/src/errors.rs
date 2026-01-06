use std::io;

/// Errors encountered during building.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(io::Error),

    /// Failed to parse the Protocol Buffers definition.
    Parser(nom::Err<nom::error::Error<String>>),

    /// Additional data remaining in the .proto file after parsing.
    TrailingGarbage(String),

    /// No input file(s) were provided.
    NoInputFiles,

    /// Provided input file does not exist.
    InputFileDoesNotExist(String),

    /// Failed to read output file.
    OutputFile(String),

    /// Failed to create output directory.
    FailedToCreateOutputDirectory(io::Error),

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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parser(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Parser(e) => write!(f, "parser error: {}", e),
            Self::TrailingGarbage(s) => {
                write!(
                    f,
                    "additional data remaining in the .proto file after parsing: {:?}",
                    s
                )
            }
            Self::NoInputFiles => write!(f, "no input files were provided"),
            Self::InputFileDoesNotExist(file) => {
                write!(f, "provided input file '{}' does not exist", file)
            }
            Self::OutputFile(file) => write!(f, "failed to read output file '{}'", file),
            Self::FailedToCreateOutputDirectory(e) => {
                write!(f, "failed to create output directory: {}", e)
            }
            Self::InvalidMessage(msg) => write!(
                f,
                "Message checks errored: {}\r\n\
                Proto definition might be invalid or something got wrong in the parsing",
                msg
            ),
            Self::InvalidImport(imp) => write!(
                f,
                "could not convert Protocol Buffers import into module import: {} (import definition might be invalid, or some characters may not be supported)",
                imp
            ),
            Self::EmptyRead => write!(
                f,
                "no messages or enums were read (definition may be invalid or only unsupported structures were defined)"
            ),
            Self::MessageOrEnumNotFound(me) => {
                write!(f, "could not find message or enum '{}'", me)
            }
            Self::InvalidDefaultEnum(en) => {
                write!(
                    f,
                    "enum field cannot be set to '{}': variant does not exist",
                    en
                )
            }
            Self::Cycle(msgs) => write!(
                f,
                "messages {:?} are cyclic (missing an optional field)",
                msgs
            ),
        }
    }
}
