//! Dialer

use runng_sys::*;
use super::*;

/// Wraps `nng_dialer`.  See [nng_dialer](https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer.5).
pub struct NngDialer {
    dialer: nng_dialer,
}

impl NngDialer {
    pub(crate) fn new(socket: nng_socket, url: &str) -> NngResult<NngDialer> {
        unsafe {
            let mut dialer = nng_dialer { id: 0 };
            let (_cstring, url) = to_cstr(url)?;
            NngFail::succeed(nng_dialer_create(&mut dialer, socket, url), NngDialer { dialer })
        }
    }
}