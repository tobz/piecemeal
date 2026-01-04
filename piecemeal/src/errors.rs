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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;
    use std::io;

    // Display tests
    #[test]
    fn test_display_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        let display = err.to_string();
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_display_utf8() {
        let bytes = vec![0xff, 0xfe];
        let utf8_err = std::str::from_utf8(&bytes).unwrap_err();
        let err = Error::Utf8(utf8_err);
        let display = err.to_string();
        assert!(display.contains("UTF-8 error"));
    }

    #[test]
    fn test_display_unexpected_end() {
        let err = Error::UnexpectedEndOfBuffer;
        assert_eq!(err.to_string(), "unexpected end of buffer");
    }

    #[test]
    fn test_display_output_too_small() {
        let err = Error::OutputBufferTooSmall;
        assert_eq!(err.to_string(), "output buffer too small");
    }

    // Error::source() tests
    #[test]
    fn test_source_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err = Error::Io(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_source_utf8() {
        let bytes = vec![0xff];
        let utf8_err = std::str::from_utf8(&bytes).unwrap_err();
        let err = Error::Utf8(utf8_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_source_none() {
        assert!(Error::UnexpectedEndOfBuffer.source().is_none());
        assert!(Error::OutputBufferTooSmall.source().is_none());
    }

    // From trait tests
    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_from_utf8_error() {
        let bytes = vec![0xff];
        let utf8_err = std::str::from_utf8(&bytes).unwrap_err();
        let err: Error = utf8_err.into();
        assert!(matches!(err, Error::Utf8(_)));
    }

    // Into<io::Error> tests
    #[test]
    fn test_into_io_error_from_io() {
        let original = io::Error::new(io::ErrorKind::NotFound, "original");
        let err = Error::Io(original);
        let io_err: io::Error = err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_into_io_error_from_utf8() {
        let bytes = vec![0xff];
        let utf8_err = std::str::from_utf8(&bytes).unwrap_err();
        let err = Error::Utf8(utf8_err);
        let io_err: io::Error = err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_into_io_error_from_other() {
        let err = Error::UnexpectedEndOfBuffer;
        let io_err: io::Error = err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::Other);

        let err = Error::OutputBufferTooSmall;
        let io_err: io::Error = err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::Other);
    }
}
