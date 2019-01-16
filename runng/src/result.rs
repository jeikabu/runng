//! Return values and error handling

use futures::sync::oneshot;
use runng_sys::*;
use std::{error, fmt, io};

/// Error values returned by NNG functions.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum NngError {
    EINTR = nng_errno_enum_NNG_EINTR as i32,
    ENOMEM = nng_errno_enum_NNG_ENOMEM as i32,
    EINVAL = nng_errno_enum_NNG_EINVAL as i32,
    EBUSY = nng_errno_enum_NNG_EBUSY as i32,
    ETIMEDOUT = nng_errno_enum_NNG_ETIMEDOUT as i32,
    ECONNREFUSED = nng_errno_enum_NNG_ECONNREFUSED as i32,
    ECLOSED = nng_errno_enum_NNG_ECLOSED as i32,
    EAGAIN = nng_errno_enum_NNG_EAGAIN as i32,
    ENOTSUP = nng_errno_enum_NNG_ENOTSUP as i32,
    EADDRINUSE = nng_errno_enum_NNG_EADDRINUSE as i32,
    ESTATE = nng_errno_enum_NNG_ESTATE as i32,
    ENOENT = nng_errno_enum_NNG_ENOENT as i32,
    EPROTO = nng_errno_enum_NNG_EPROTO as i32,
    EUNREACHABLE = nng_errno_enum_NNG_EUNREACHABLE as i32,
    EADDRINVAL = nng_errno_enum_NNG_EADDRINVAL as i32,
    EPERM = nng_errno_enum_NNG_EPERM as i32,
    EMSGSIZE = nng_errno_enum_NNG_EMSGSIZE as i32,
    ECONNABORTED = nng_errno_enum_NNG_ECONNABORTED as i32,
    ECONNRESET = nng_errno_enum_NNG_ECONNRESET as i32,
    ECANCELED = nng_errno_enum_NNG_ECANCELED as i32,
    ENOFILES = nng_errno_enum_NNG_ENOFILES as i32,
    ENOSPC = nng_errno_enum_NNG_ENOSPC as i32,
    EEXIST = nng_errno_enum_NNG_EEXIST as i32,
    EREADONLY = nng_errno_enum_NNG_EREADONLY as i32,
    EWRITEONLY = nng_errno_enum_NNG_EWRITEONLY as i32,
    ECRYPTO = nng_errno_enum_NNG_ECRYPTO as i32,
    EPEERAUTH = nng_errno_enum_NNG_EPEERAUTH as i32,
    ENOARG = nng_errno_enum_NNG_ENOARG as i32,
    EAMBIGUOUS = nng_errno_enum_NNG_EAMBIGUOUS as i32,
    EBADTYPE = nng_errno_enum_NNG_EBADTYPE as i32,
    EINTERNAL = nng_errno_enum_NNG_EINTERNAL as i32,
    ESYSERR = nng_errno_enum_NNG_ESYSERR as i32,
    ETRANERR = nng_errno_enum_NNG_ETRANERR as i32,
}

impl NngError {
    /// Converts value returned by NNG method into `error::Error`.
    #[allow(clippy::cyclomatic_complexity)]
    pub fn from_i32(value: i32) -> Option<NngError> {
        match value {
            value if value == NngError::EINTR as i32 => Some(NngError::EINTR),
            value if value == NngError::ENOMEM as i32 => Some(NngError::ENOMEM),
            value if value == NngError::EINVAL as i32 => Some(NngError::EINVAL),
            value if value == NngError::EBUSY as i32 => Some(NngError::EBUSY),
            value if value == NngError::ETIMEDOUT as i32 => Some(NngError::ETIMEDOUT),
            value if value == NngError::ECONNREFUSED as i32 => Some(NngError::ECONNREFUSED),
            value if value == NngError::ECLOSED as i32 => Some(NngError::ECLOSED),
            value if value == NngError::EAGAIN as i32 => Some(NngError::EAGAIN),
            value if value == NngError::ENOTSUP as i32 => Some(NngError::ENOTSUP),
            value if value == NngError::EADDRINUSE as i32 => Some(NngError::EADDRINUSE),
            value if value == NngError::ESTATE as i32 => Some(NngError::ESTATE),
            value if value == NngError::ENOENT as i32 => Some(NngError::ENOENT),
            value if value == NngError::EPROTO as i32 => Some(NngError::EPROTO),
            value if value == NngError::EUNREACHABLE as i32 => Some(NngError::EUNREACHABLE),
            value if value == NngError::EADDRINVAL as i32 => Some(NngError::EADDRINVAL),
            value if value == NngError::EPERM as i32 => Some(NngError::EPERM),
            value if value == NngError::EMSGSIZE as i32 => Some(NngError::EMSGSIZE),
            value if value == NngError::ECONNABORTED as i32 => Some(NngError::ECONNABORTED),
            value if value == NngError::ECONNRESET as i32 => Some(NngError::ECONNRESET),
            value if value == NngError::ECANCELED as i32 => Some(NngError::ECANCELED),
            value if value == NngError::ENOFILES as i32 => Some(NngError::ENOFILES),
            value if value == NngError::ENOSPC as i32 => Some(NngError::ENOSPC),
            value if value == NngError::EEXIST as i32 => Some(NngError::EEXIST),
            value if value == NngError::EREADONLY as i32 => Some(NngError::EREADONLY),
            value if value == NngError::EWRITEONLY as i32 => Some(NngError::EWRITEONLY),
            value if value == NngError::ECRYPTO as i32 => Some(NngError::ECRYPTO),
            value if value == NngError::EPEERAUTH as i32 => Some(NngError::EPEERAUTH),
            value if value == NngError::ENOARG as i32 => Some(NngError::ENOARG),
            value if value == NngError::EAMBIGUOUS as i32 => Some(NngError::EAMBIGUOUS),
            value if value == NngError::EBADTYPE as i32 => Some(NngError::EBADTYPE),
            value if value == NngError::EINTERNAL as i32 => Some(NngError::EINTERNAL),
            value if value == NngError::ESYSERR as i32 => Some(NngError::ESYSERR),
            value if value == NngError::ETRANERR as i32 => Some(NngError::ETRANERR),

            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum NngFail {
    Err(NngError),
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
        } else if let Some(error) = NngError::from_i32(value) {
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

impl error::Error for NngError {}

impl fmt::Display for NngError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
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
