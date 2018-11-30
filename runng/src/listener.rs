//! Listener

use runng_sys::*;
use std::sync::Arc;
use runng_derive::{NngGetOpts, NngSetOpts};
use super::*;

/// Wraps `nng_listener`.  See [nng_listener](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener.5).
#[derive(NngGetOpts, NngSetOpts)]
#[prefix = "nng_listener_"]
pub struct NngListener {
    #[nng_member]
    listener: nng_listener,
    socket: Arc<NngSocket>
}

impl NngListener {
    /// See [nng_listener_create](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_create.3).
    pub(crate) fn new(socket: Arc<NngSocket>, url: &str) -> NngResult<NngListener> {
        unsafe {
            let mut listener = nng_listener { id: 0 };
            let (_cstring, url) = to_cstr(url)?;
            NngFail::succeed(
                nng_listener_create(&mut listener, socket.nng_socket(), url), 
                NngListener { listener, socket }
            )
        }
    }

    /// See [nng_listener_start](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_start.3).
    pub fn start(&self/*, flags: i32*/) -> NngReturn {
        // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
        // be set before the dialer is started.
        unsafe {
            NngFail::from_i32(nng_listener_start(self.listener, 0))
        }
    }
}