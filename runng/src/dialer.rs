//! Dialer

use super::*;
use runng_derive::{NngGetOpts, NngSetOpts};
use runng_sys::*;

/// Wraps `nng_dialer`.  See [nng_dialer](https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer.5).
#[derive(NngGetOpts, NngSetOpts)]
#[prefix = "nng_dialer_"]
pub struct NngDialer {
    #[nng_member]
    dialer: nng_dialer,
    socket: NngSocket,
}

impl NngDialer {
    /// See [nng_dialer_create](https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer_create.3).
    pub(crate) fn create(socket: NngSocket, url: &str) -> NngResult<Self> {
        unsafe {
            let mut dialer = nng_dialer { id: 0 };
            let (_cstring, url) = to_cstr(url)?;
            NngFail::succeed(
                nng_dialer_create(&mut dialer, socket.nng_socket(), url),
                NngDialer { dialer, socket },
            )
        }
    }

    // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
    // be set before the dialer is started.
    pub fn start(&self) -> NngReturn {
        unsafe { NngFail::from_i32(nng_dialer_start(self.dialer, 0)) }
    }
}

/// "Unsafe" version of `NngDialer`.  Merely wraps `nng_dialer` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
pub struct UnsafeDialer {
    dialer: nng_dialer,
}

impl UnsafeDialer {
    pub fn new(dialer: nng_dialer) -> Self {
        Self { dialer }
    }

    pub fn id(&self) -> i32 {
        unsafe { nng_dialer_id(self.dialer) }
    }
}
