//! Return values and error handling

use futures::sync::oneshot;
use runng_sys::*;
use std::{error, fmt, io};

#[derive(Debug)]
pub enum NngFail {
    Err(nng_errno_enum),
    Unknown(i32),
    IoError(io::Error),
    NulError(std::ffi::NulError),
    Unit,
    Canceled,
    NoneError,
}

impl NngFail {
    /// Converts values returned by NNG methods into `Result<>`
    pub fn from_i32(value: i32) -> NngReturn {
        if value == 0 {
            Ok(())
        } else if let Some(error) = nng_errno_enum::from_i32(value) {
            Err(NngFail::Err(error))
        } else {
            Err(NngFail::Unknown(value))
        }
    }
    pub fn succeed<T>(value: i32, result: T) -> NngResult<T> {
        match NngFail::from_i32(value) {
            Ok(()) => Ok(result),
            Err(error) => Err(error),
        }
    }
    pub fn succeed_then<T, F: FnOnce() -> T>(value: i32, result: F) -> NngResult<T> {
        match NngFail::from_i32(value) {
            Ok(()) => Ok(result()),
            Err(error) => Err(error),
        }
    }
}

impl From<io::Error> for NngFail {
    fn from(err: io::Error) -> NngFail {
        NngFail::IoError(err)
    }
}

impl From<std::ffi::NulError> for NngFail {
    fn from(err: std::ffi::NulError) -> NngFail {
        NngFail::NulError(err)
    }
}

impl From<()> for NngFail {
    fn from(_: ()) -> NngFail {
        NngFail::Unit
    }
}

impl From<oneshot::Canceled> for NngFail {
    fn from(_: oneshot::Canceled) -> NngFail {
        NngFail::Canceled
    }
}

impl From<NngFail> for io::Error {
    fn from(_err: NngFail) -> io::Error {
        io::Error::from(io::ErrorKind::Other)
    }
}

pub type NngResult<T> = Result<T, NngFail>;
pub type NngReturn = NngResult<()>;
