use deku::DekuError;
use std::error;
use std::fmt;
use std::io;
use std::result;

pub type RatmapResult<T> = result::Result<T, RatmapError>;

#[derive(Debug)]
pub enum RatmapError {
    /// I/O error
    Io(io::Error),

    /// Deku error
    Deku(DekuError),
}

impl fmt::Display for RatmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RatmapError::Io(err) => write!(f, "{}", err),
            RatmapError::Deku(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for RatmapError {}

impl From<io::Error> for RatmapError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<DekuError> for RatmapError {
    fn from(e: DekuError) -> Self {
        Self::Deku(e)
    }
}
