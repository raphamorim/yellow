use std::fmt;
use std::io;

/// Result type for Yellow operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during terminal operations
#[derive(Debug)]
pub enum Error {
    /// I/O error occurred
    Io(io::Error),
    /// Terminal is not initialized
    NotInitialized,
    /// Terminal is already initialized
    AlreadyInitialized,
    /// Invalid color pair ID
    InvalidColorPair(u8),
    /// Invalid coordinates
    InvalidCoordinates { y: u16, x: u16 },
    /// Invalid window dimensions
    InvalidDimensions { height: u16, width: u16 },
    /// Operation not supported on this platform
    NotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::NotInitialized => write!(f, "Terminal not initialized"),
            Error::AlreadyInitialized => write!(f, "Terminal already initialized"),
            Error::InvalidColorPair(id) => write!(f, "Invalid color pair ID: {}", id),
            Error::InvalidCoordinates { y, x } => {
                write!(f, "Invalid coordinates: ({}, {})", y, x)
            }
            Error::InvalidDimensions { height, width } => {
                write!(f, "Invalid dimensions: {}x{}", height, width)
            }
            Error::NotSupported => write!(f, "Operation not supported"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::Other, "fmt error"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NotInitialized;
        assert_eq!(err.to_string(), "Terminal not initialized");

        let err = Error::InvalidColorPair(5);
        assert_eq!(err.to_string(), "Invalid color pair ID: 5");

        let err = Error::InvalidCoordinates { y: 10, x: 20 };
        assert_eq!(err.to_string(), "Invalid coordinates: (10, 20)");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test error");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
