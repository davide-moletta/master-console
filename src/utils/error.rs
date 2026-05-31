use std::{error::Error, fmt, result::Result};

/// Custom error type used to convert specific error types to custom ones
#[derive(Debug)]
pub enum GBError {
    Io(std::io::Error),
    OversizedROM,
}

/// Custom [`Result`] that returns a [`GBError`]
pub type GBResult<T> = Result<T, GBError>;

/// [`fmt::Display`] implementation for [`GBError`]
impl fmt::Display for GBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO Error: {}", e),
            Self::OversizedROM => write!(f, "Input ROM is too big"),
        }
    }
}

/// [`Error`] implementation for [`GBError`]
impl Error for GBError {}

/// [`From`] implementation for [`GBError`]
impl From<std::io::Error> for GBError {
    fn from(err: std::io::Error) -> Self {
        GBError::Io(err)
    }
}
