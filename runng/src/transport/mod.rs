//! NNG transports.  See [Section 7](https://nanomsg.github.io/nng/man/v1.1.0/index.html#_section_7_protocols_and_transports).

use crate::*;

pub fn nng_inproc_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_inproc_register()) }
}

pub fn nng_ipc_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_ipc_register()) }
}

pub fn nng_tcp_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_tcp_register()) }
}

pub fn nng_tls_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_tls_register()) }
}

pub fn nng_wss_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_wss_register()) }
}

pub fn nng_ws_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_ws_register()) }
}

pub fn nng_zt_register() -> Result<()> {
    unsafe { nng_int_to_result(runng_sys::nng_zt_register()) }
}
