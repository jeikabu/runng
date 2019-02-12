//! Return values and error handling

use futures::sync::oneshot;
use runng_sys::*;
use std::{error, fmt, io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Errno(nng_errno_enum),
    UnknownErrno(i32),
    IoError(io::Error),
    NulError(std::ffi::NulError),
    Unit,
    Canceled(oneshot::Canceled),
}

impl Error {
    /// Converts values returned by NNG methods into `Result<>`
    pub fn from_i32(value: i32) -> Result<()> {
        if value == 0 {
            Ok(())
        } else if let Ok(error) = Error::try_from(value) {
            Err(error)
        } else {
            Err(Error::UnknownErrno(value))
        }
    }
    // TODO: replace this with std::num::TryFromIntError once stabilized:
    // https://doc.rust-lang.org/std/convert/trait.TryFrom.html
    fn try_from(value: i32) -> result::Result<Self, TryFromIntError> {
        if value == 0 {
            Err(TryFromIntError)
        } else if let Ok(error) = nng_errno_enum::try_from(value) {
            Ok(Error::Errno(error))
        } else {
            Ok(Error::UnknownErrno(value))
        }
    }
    /// If `value` is zero returns `Ok(result())`.  Otherwise converts `value` to an `Error` and returns that.
    pub fn zero_map<T, F: FnOnce() -> T>(value: i32, result: F) -> Result<T> {
        Error::from_i32(value).map(|_| result())
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Errno(ref err) => write!(f, "{:?}", err),
            UnknownErrno(ref err) => err.fmt(f),
            IoError(ref err) => err.fmt(f),
            NulError(ref err) => err.fmt(f),
            Unit => write!(f, "()"),
            Canceled(ref err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Error {
        Error::NulError(err)
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::Unit
    }
}

impl From<oneshot::Canceled> for Error {
    fn from(err: oneshot::Canceled) -> Error {
        Error::Canceled(err)
    }
}
