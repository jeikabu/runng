//! NNG transports.  See [Section 7](https://nanomsg.github.io/nng/man/v1.1.0/index.html#_section_7_protocols_and_transports).

use super::*;

pub fn nng_inproc_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_inproc_register()) }
}

pub fn nng_ipc_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_ipc_register()) }
}

pub fn nng_tcp_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_tcp_register()) }
}

pub fn nng_tls_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_tls_register()) }
}

pub fn nng_wss_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_wss_register()) }
}

pub fn nng_ws_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_ws_register()) }
}

pub fn nng_zt_register() -> NngReturn {
    unsafe { NngFail::from_i32(runng_sys::nng_zt_register()) }
}
