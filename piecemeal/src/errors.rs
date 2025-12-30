//! Error handling.

/// Main error type when encoding.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(std::io::Error),

    /// UTF-8 error.
    Utf8(std::str::Utf8Error),

    /// Unexpectedly ran out of data when reading/writing to a byte buffer.
    UnexpectedEndOfBuffer,

    /// The supplied output buffer is not large enough to serialize the message.
    OutputBufferTooSmall,
}

/// A wrapper for `Result<T, Error>`
pub type ProtoResult<T> = ::core::result::Result<T, Error>;

impl From<Error> for std::io::Error {
    fn from(val: Error) -> Self {
        match val {
            Error::Io(x) => x,
            Error::Utf8(x) => std::io::Error::new(std::io::ErrorKind::InvalidData, x),
            x => std::io::Error::other(x),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Error {
        Error::Utf8(e)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Utf8(e) => write!(f, "UTF-8 error: {}", e),
            Error::UnexpectedEndOfBuffer => write!(f, "unexpected end of buffer"),
            Error::OutputBufferTooSmall => write!(f, "output buffer too small"),
        }
    }
}
