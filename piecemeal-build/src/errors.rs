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
