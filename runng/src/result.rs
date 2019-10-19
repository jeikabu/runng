//! Return values and error handling

use futures::sync::oneshot;
use runng_sys::*;
use std::{error, fmt, io, result};

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
        } else if let Ok(error) = NngErrno::try_from(value) {
            Err(NngFail::Err(error as u32))
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

use std::convert::TryFrom;

type NewResult<T> = result::Result<T, Error>;

/// Converts integers returned by NNG methods into `Result`.
/// 0 is Ok() and anything else is Err()
pub fn nng_int_to_result(value: i32) -> NewResult<()> {
    if value == 0 {
        Ok(())
    } else if let Ok(error) = Error::try_from(value) {
        Err(error)
    } else {
        Err(Error::UnknownErrno(value))
    }
}

/// Error values returned by NNG functions.
/// The special errno flags NNG_ESYSERR/NNG_ETRANERR are represented by Error::SysErr() and Error::TranErr()
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum NngErrno {
    EINTR = runng_sys::NNG_EINTR as i32,
    ENOMEM = runng_sys::NNG_ENOMEM as i32,
    EINVAL = runng_sys::NNG_EINVAL as i32,
    EBUSY = runng_sys::NNG_EBUSY as i32,
    ETIMEDOUT = runng_sys::NNG_ETIMEDOUT as i32,
    ECONNREFUSED = runng_sys::NNG_ECONNREFUSED as i32,
    ECLOSED = runng_sys::NNG_ECLOSED as i32,
    EAGAIN = runng_sys::NNG_EAGAIN as i32,
    ENOTSUP = runng_sys::NNG_ENOTSUP as i32,
    EADDRINUSE = runng_sys::NNG_EADDRINUSE as i32,
    ESTATE = runng_sys::NNG_ESTATE as i32,
    ENOENT = runng_sys::NNG_ENOENT as i32,
    EPROTO = runng_sys::NNG_EPROTO as i32,
    EUNREACHABLE = runng_sys::NNG_EUNREACHABLE as i32,
    EADDRINVAL = runng_sys::NNG_EADDRINVAL as i32,
    EPERM = runng_sys::NNG_EPERM as i32,
    EMSGSIZE = runng_sys::NNG_EMSGSIZE as i32,
    ECONNABORTED = runng_sys::NNG_ECONNABORTED as i32,
    ECONNRESET = runng_sys::NNG_ECONNRESET as i32,
    ECANCELED = runng_sys::NNG_ECANCELED as i32,
    ENOFILES = runng_sys::NNG_ENOFILES as i32,
    ENOSPC = runng_sys::NNG_ENOSPC as i32,
    EEXIST = runng_sys::NNG_EEXIST as i32,
    EREADONLY = runng_sys::NNG_EREADONLY as i32,
    EWRITEONLY = runng_sys::NNG_EWRITEONLY as i32,
    ECRYPTO = runng_sys::NNG_ECRYPTO as i32,
    EPEERAUTH = runng_sys::NNG_EPEERAUTH as i32,
    ENOARG = runng_sys::NNG_ENOARG as i32,
    EAMBIGUOUS = runng_sys::NNG_EAMBIGUOUS as i32,
    EBADTYPE = runng_sys::NNG_EBADTYPE as i32,
    EINTERNAL = runng_sys::NNG_EINTERNAL as i32,
    // ESYSERR(int),
    // ETRANERR(int),
}

impl TryFrom<i32> for NngErrno {
    type Error = TryFromIntError;

    #[allow(clippy::cyclomatic_complexity)]
    fn try_from(value: i32) -> result::Result<Self, Self::Error> {
        match value {
            value if value == NngErrno::EINTR as i32 => Ok(NngErrno::EINTR),
            value if value == NngErrno::ENOMEM as i32 => Ok(NngErrno::ENOMEM),
            value if value == NngErrno::EINVAL as i32 => Ok(NngErrno::EINVAL),
            value if value == NngErrno::EBUSY as i32 => Ok(NngErrno::EBUSY),
            value if value == NngErrno::ETIMEDOUT as i32 => Ok(NngErrno::ETIMEDOUT),
            value if value == NngErrno::ECONNREFUSED as i32 => Ok(NngErrno::ECONNREFUSED),
            value if value == NngErrno::ECLOSED as i32 => Ok(NngErrno::ECLOSED),
            value if value == NngErrno::EAGAIN as i32 => Ok(NngErrno::EAGAIN),
            value if value == NngErrno::ENOTSUP as i32 => Ok(NngErrno::ENOTSUP),
            value if value == NngErrno::EADDRINUSE as i32 => Ok(NngErrno::EADDRINUSE),
            value if value == NngErrno::ESTATE as i32 => Ok(NngErrno::ESTATE),
            value if value == NngErrno::ENOENT as i32 => Ok(NngErrno::ENOENT),
            value if value == NngErrno::EPROTO as i32 => Ok(NngErrno::EPROTO),
            value if value == NngErrno::EUNREACHABLE as i32 => Ok(NngErrno::EUNREACHABLE),
            value if value == NngErrno::EADDRINVAL as i32 => Ok(NngErrno::EADDRINVAL),
            value if value == NngErrno::EPERM as i32 => Ok(NngErrno::EPERM),
            value if value == NngErrno::EMSGSIZE as i32 => Ok(NngErrno::EMSGSIZE),
            value if value == NngErrno::ECONNABORTED as i32 => Ok(NngErrno::ECONNABORTED),
            value if value == NngErrno::ECONNRESET as i32 => Ok(NngErrno::ECONNRESET),
            value if value == NngErrno::ECANCELED as i32 => Ok(NngErrno::ECANCELED),
            value if value == NngErrno::ENOFILES as i32 => Ok(NngErrno::ENOFILES),
            value if value == NngErrno::ENOSPC as i32 => Ok(NngErrno::ENOSPC),
            value if value == NngErrno::EEXIST as i32 => Ok(NngErrno::EEXIST),
            value if value == NngErrno::EREADONLY as i32 => Ok(NngErrno::EREADONLY),
            value if value == NngErrno::EWRITEONLY as i32 => Ok(NngErrno::EWRITEONLY),
            value if value == NngErrno::ECRYPTO as i32 => Ok(NngErrno::ECRYPTO),
            value if value == NngErrno::EPEERAUTH as i32 => Ok(NngErrno::EPEERAUTH),
            value if value == NngErrno::ENOARG as i32 => Ok(NngErrno::ENOARG),
            value if value == NngErrno::EAMBIGUOUS as i32 => Ok(NngErrno::EAMBIGUOUS),
            value if value == NngErrno::EBADTYPE as i32 => Ok(NngErrno::EBADTYPE),
            value if value == NngErrno::EINTERNAL as i32 => Ok(NngErrno::EINTERNAL),
            _ => Err(TryFromIntError),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Errno(NngErrno),
    /// NNG_ESYSERR
    SysErr(i32),
    /// NNG_ETRANERR
    TranErr(i32),
    UnknownErrno(i32),
    NulError(std::ffi::NulError),
    Unit,
    Canceled(oneshot::Canceled),
}

impl Error {
    /// If `value` is zero returns `Ok(result())`.  Otherwise converts `value` to an `Error` and returns that.
    pub fn zero_map<T, F: FnOnce() -> T>(value: i32, result: F) -> NewResult<T> {
        nng_int_to_result(value).map(|_| result())
    }
}

impl TryFrom<i32> for Error {
    type Error = TryFromIntError;

    fn try_from(value: i32) -> result::Result<Self, Self::Error> {
        const ESYSERR: i32 = runng_sys::NNG_ESYSERR as i32;
        const ETRANERR: i32 = runng_sys::NNG_ETRANERR as i32;
        if value == 0 {
            Err(TryFromIntError)
        } else if let Ok(error) = NngErrno::try_from(value) {
            Ok(Error::Errno(error))
        } else if value & ESYSERR != 0 {
            Ok(Error::SysErr(value ^ ESYSERR))
        } else if value & ETRANERR != 0 {
            Ok(Error::SysErr(value ^ ETRANERR))
        } else {
            Ok(Error::UnknownErrno(value))
        }
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Errno(ref err) => write!(f, "{:?}", err),
            SysErr(ref err) => write!(f, "ESYSERR({})", err),
            TranErr(ref err) => write!(f, "ETRANERR({})", err),
            UnknownErrno(ref err) => err.fmt(f),
            NulError(ref err) => err.fmt(f),
            Unit => write!(f, "()"),
            Canceled(ref err) => err.fmt(f),
        }
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
