//! Return values and error handling

use futures::channel::oneshot;
use runng_sys::*;
use std::convert::TryFrom;
use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Error>;

/// Flattens nested results.
/// Primary use case is with channels:
/// ```
/// use futures::channel::oneshot;
/// use futures_util::future::FutureExt; // for map()
///
/// // Wrapper to explicity show the types
/// fn flatten<T>(input: T) -> impl futures::Future<Output = runng::Result<runng::msg::NngMsg>>
/// where
///     T: futures::Future<Output = Result<runng::Result<runng::msg::NngMsg>, oneshot::Canceled>>,
/// {
///     input.map(runng::flatten_result)
/// }
///
/// let (_, receiver) = oneshot::channel();
/// let receiver = flatten(receiver);
/// ```
pub fn flatten_result<T, E, F>(
    result: result::Result<result::Result<T, E>, F>,
) -> result::Result<T, E>
where
    E: std::convert::From<F>,
{
    match result {
        Ok(result) => result,
        Err(err) => Err(err.into()),
    }
}

/// Converts integers returned by NNG methods into `Result`.
/// 0 is Ok() and anything else is Err()
pub fn nng_int_to_result(value: i32) -> Result<()> {
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
    ECONNSHUT = runng_sys::NNG_ECONNSHUT as i32,
    EINTERNAL = runng_sys::NNG_EINTERNAL as i32,
    // ESYSERR(int),
    // ETRANERR(int),
}

impl TryFrom<i32> for NngErrno {
    type Error = EnumFromIntError;

    fn try_from(value: i32) -> result::Result<Self, Self::Error> {
        match value as u32 {
            runng_sys::NNG_EINTR => Ok(NngErrno::EINTR),
            runng_sys::NNG_ENOMEM => Ok(NngErrno::ENOMEM),
            runng_sys::NNG_EINVAL => Ok(NngErrno::EINVAL),
            runng_sys::NNG_EBUSY => Ok(NngErrno::EBUSY),
            runng_sys::NNG_ETIMEDOUT => Ok(NngErrno::ETIMEDOUT),
            runng_sys::NNG_ECONNREFUSED => Ok(NngErrno::ECONNREFUSED),
            runng_sys::NNG_ECLOSED => Ok(NngErrno::ECLOSED),
            runng_sys::NNG_EAGAIN => Ok(NngErrno::EAGAIN),
            runng_sys::NNG_ENOTSUP => Ok(NngErrno::ENOTSUP),
            runng_sys::NNG_EADDRINUSE => Ok(NngErrno::EADDRINUSE),
            runng_sys::NNG_ESTATE => Ok(NngErrno::ESTATE),
            runng_sys::NNG_ENOENT => Ok(NngErrno::ENOENT),
            runng_sys::NNG_EPROTO => Ok(NngErrno::EPROTO),
            runng_sys::NNG_EUNREACHABLE => Ok(NngErrno::EUNREACHABLE),
            runng_sys::NNG_EADDRINVAL => Ok(NngErrno::EADDRINVAL),
            runng_sys::NNG_EPERM => Ok(NngErrno::EPERM),
            runng_sys::NNG_EMSGSIZE => Ok(NngErrno::EMSGSIZE),
            runng_sys::NNG_ECONNABORTED => Ok(NngErrno::ECONNABORTED),
            runng_sys::NNG_ECONNRESET => Ok(NngErrno::ECONNRESET),
            runng_sys::NNG_ECANCELED => Ok(NngErrno::ECANCELED),
            runng_sys::NNG_ENOFILES => Ok(NngErrno::ENOFILES),
            runng_sys::NNG_ENOSPC => Ok(NngErrno::ENOSPC),
            runng_sys::NNG_EEXIST => Ok(NngErrno::EEXIST),
            runng_sys::NNG_EREADONLY => Ok(NngErrno::EREADONLY),
            runng_sys::NNG_EWRITEONLY => Ok(NngErrno::EWRITEONLY),
            runng_sys::NNG_ECRYPTO => Ok(NngErrno::ECRYPTO),
            runng_sys::NNG_EPEERAUTH => Ok(NngErrno::EPEERAUTH),
            runng_sys::NNG_ENOARG => Ok(NngErrno::ENOARG),
            runng_sys::NNG_EAMBIGUOUS => Ok(NngErrno::EAMBIGUOUS),
            runng_sys::NNG_EBADTYPE => Ok(NngErrno::EBADTYPE),
            runng_sys::NNG_ECONNSHUT => Ok(NngErrno::ECONNSHUT),
            runng_sys::NNG_EINTERNAL => Ok(NngErrno::EINTERNAL),
            _ => Err(EnumFromIntError(value)),
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
    TryFromError(i32),
}

impl Error {
    /// If `value` is zero returns `Ok(result())`.  Otherwise converts `value` to an `Error` and returns that.
    pub fn zero_map<T, F: FnOnce() -> T>(value: i32, result: F) -> Result<T> {
        nng_int_to_result(value).map(|_| result())
    }
}

impl TryFrom<i32> for Error {
    type Error = EnumFromIntError;

    fn try_from(value: i32) -> result::Result<Self, Self::Error> {
        const ESYSERR: i32 = runng_sys::NNG_ESYSERR as i32;
        const ETRANERR: i32 = runng_sys::NNG_ETRANERR as i32;
        if value == 0 {
            Err(EnumFromIntError(value))
        } else if let Ok(error) = NngErrno::try_from(value) {
            Ok(Error::Errno(error))
        } else if value & ESYSERR != 0 {
            Ok(Error::SysErr(value ^ ESYSERR))
        } else if value & ETRANERR != 0 {
            Ok(Error::TranErr(value ^ ETRANERR))
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
            TryFromError(value) => write!(f, "EnumFromIntError({})", value),
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

impl From<EnumFromIntError> for Error {
    fn from(err: EnumFromIntError) -> Error {
        Error::TryFromError(err.0)
    }
}
