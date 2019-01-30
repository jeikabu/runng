// Suppress the flurry of warnings caused by using "C" naming conventions
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Disable clippy since this is all bindgen generated code
#![allow(clippy::all)]
#![no_std]

// This matches bindgen::Builder output
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "legacy-111-rc4"))]
impl nng_stat_type_enum {
    /// Converts value returned by [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3) into `nng_stat_type_enum`.
    pub fn from_i32(value: i32) -> Option<nng_stat_type_enum> {
        use nng_stat_type_enum::*;
        match value {
            value if value == NNG_STAT_SCOPE as i32 => Some(NNG_STAT_SCOPE),
            value if value == NNG_STAT_LEVEL as i32 => Some(NNG_STAT_LEVEL),
            value if value == NNG_STAT_COUNTER as i32 => Some(NNG_STAT_COUNTER),
            value if value == NNG_STAT_STRING as i32 => Some(NNG_STAT_STRING),
            value if value == NNG_STAT_BOOLEAN as i32 => Some(NNG_STAT_BOOLEAN),
            value if value == NNG_STAT_ID as i32 => Some(NNG_STAT_ID),
            _ => None,
        }
    }
}

#[cfg(not(feature = "legacy-111-rc4"))]
impl nng_unit_enum {
    /// Converts value returned by [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3) into `nng_unit_enum`.
    pub fn from_i32(value: i32) -> Option<nng_unit_enum> {
        use nng_unit_enum::*;
        match value {
            value if value == NNG_UNIT_NONE as i32 => Some(NNG_UNIT_NONE),
            value if value == NNG_UNIT_BYTES as i32 => Some(NNG_UNIT_BYTES),
            value if value == NNG_UNIT_MESSAGES as i32 => Some(NNG_UNIT_MESSAGES),
            value if value == NNG_UNIT_MILLIS as i32 => Some(NNG_UNIT_MILLIS),
            value if value == NNG_UNIT_EVENTS as i32 => Some(NNG_UNIT_EVENTS),
            _ => None,
        }
    }
}

#[cfg(not(feature = "legacy-111-rc4"))]
impl nng_errno_enum {
    /// Converts value returned by NNG method into `error::Error`.
    #[allow(clippy::cyclomatic_complexity)]
    pub fn from_i32(value: i32) -> Option<nng_errno_enum> {
        use nng_errno_enum::*;
        match value {
            value if value == NNG_EINTR as i32 => Some(NNG_EINTR),
            value if value == NNG_ENOMEM as i32 => Some(NNG_ENOMEM),
            value if value == NNG_EINVAL as i32 => Some(NNG_EINVAL),
            value if value == NNG_EBUSY as i32 => Some(NNG_EBUSY),
            value if value == NNG_ETIMEDOUT as i32 => Some(NNG_ETIMEDOUT),
            value if value == NNG_ECONNREFUSED as i32 => Some(NNG_ECONNREFUSED),
            value if value == NNG_ECLOSED as i32 => Some(NNG_ECLOSED),
            value if value == NNG_EAGAIN as i32 => Some(NNG_EAGAIN),
            value if value == NNG_ENOTSUP as i32 => Some(NNG_ENOTSUP),
            value if value == NNG_EADDRINUSE as i32 => Some(NNG_EADDRINUSE),
            value if value == NNG_ESTATE as i32 => Some(NNG_ESTATE),
            value if value == NNG_ENOENT as i32 => Some(NNG_ENOENT),
            value if value == NNG_EPROTO as i32 => Some(NNG_EPROTO),
            value if value == NNG_EUNREACHABLE as i32 => Some(NNG_EUNREACHABLE),
            value if value == NNG_EADDRINVAL as i32 => Some(NNG_EADDRINVAL),
            value if value == NNG_EPERM as i32 => Some(NNG_EPERM),
            value if value == NNG_EMSGSIZE as i32 => Some(NNG_EMSGSIZE),
            value if value == NNG_ECONNABORTED as i32 => Some(NNG_ECONNABORTED),
            value if value == NNG_ECONNRESET as i32 => Some(NNG_ECONNRESET),
            value if value == NNG_ECANCELED as i32 => Some(NNG_ECANCELED),
            value if value == NNG_ENOFILES as i32 => Some(NNG_ENOFILES),
            value if value == NNG_ENOSPC as i32 => Some(NNG_ENOSPC),
            value if value == NNG_EEXIST as i32 => Some(NNG_EEXIST),
            value if value == NNG_EREADONLY as i32 => Some(NNG_EREADONLY),
            value if value == NNG_EWRITEONLY as i32 => Some(NNG_EWRITEONLY),
            value if value == NNG_ECRYPTO as i32 => Some(NNG_ECRYPTO),
            value if value == NNG_EPEERAUTH as i32 => Some(NNG_EPEERAUTH),
            value if value == NNG_ENOARG as i32 => Some(NNG_ENOARG),
            value if value == NNG_EAMBIGUOUS as i32 => Some(NNG_EAMBIGUOUS),
            value if value == NNG_EBADTYPE as i32 => Some(NNG_EBADTYPE),
            value if value == NNG_EINTERNAL as i32 => Some(NNG_EINTERNAL),
            value if value == NNG_ESYSERR as i32 => Some(NNG_ESYSERR),
            value if value == NNG_ETRANERR as i32 => Some(NNG_ETRANERR),

            _ => None,
        }
    }
}

#[cfg(not(feature = "legacy-111-rc4"))]
impl nng_pipe_ev {
    pub fn from_i32(value: i32) -> Option<nng_pipe_ev> {
        use nng_pipe_ev::*;
        match value {
            value if value == NNG_PIPE_EV_ADD_PRE as i32 => Some(NNG_PIPE_EV_ADD_PRE),
            value if value == NNG_PIPE_EV_ADD_POST as i32 => Some(NNG_PIPE_EV_ADD_POST),
            value if value == NNG_PIPE_EV_REM_POST as i32 => Some(NNG_PIPE_EV_REM_POST),
            _ => None,
        }
    }
}
