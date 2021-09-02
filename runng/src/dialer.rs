//! Dialers connect to listeners.

use crate::*;
use runng_derive::{NngGetOpts, NngSetOpts};
use runng_sys::*;

/// Wraps `nng_dialer`.  See [nng_dialer](https://nng.nanomsg.org/man/v1.2.2/nng_dialer.5).
#[derive(Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_dialer_"]
pub struct NngDialer {
    dialer: nng_dialer,
    socket: NngSocket,
}

impl NngDialer {
    /// See [nng_dialer_create](https://nng.nanomsg.org/man/v1.2.2/nng_dialer_create.3).
    pub(crate) fn new(socket: NngSocket, url: &str) -> Result<Self> {
        unsafe {
            let mut dialer = nng_dialer::default();
            let (_cstring, url) = to_cstr(url)?;
            let res = nng_dialer_create(&mut dialer, socket.nng_socket(), url);
            nng_int_to_result(res).map(|_| NngDialer { dialer, socket })
        }
    }

    // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
    // be set before the dialer is started.
    pub fn start(&self) -> Result<()> {
        unsafe { nng_int_to_result(nng_dialer_start(self.dialer, 0)) }
    }

    pub fn get_sockaddr(&self, option: NngOption) -> Result<SockAddr> {
        unsafe {
            let mut sockaddr = nng_sockaddr::default();
            nng_int_to_result(nng_dialer_getopt_sockaddr(
                self.dialer,
                option.as_cptr(),
                &mut sockaddr,
            ))
            .and_then(|_| SockAddr::try_from(sockaddr))
        }
    }
}

impl NngWrapper for NngDialer {
    type NngType = nng_dialer;
    unsafe fn get_nng_type(&self) -> Self::NngType {
        self.dialer
    }
}

/// "Unsafe" version of `NngDialer`.  Merely wraps `nng_dialer` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
#[derive(Debug)]
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
