//! Listener

use super::*;
use runng_derive::{NngGetOpts, NngSetOpts};
use runng_sys::*;
use std::sync::Arc;

/// Wraps `nng_listener`.  See [nng_listener](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener.5).
#[derive(NngGetOpts, NngSetOpts)]
#[prefix = "nng_listener_"]
pub struct NngListener {
    #[nng_member]
    listener: nng_listener,
    socket: Arc<NngSocket>,
}

impl NngListener {
    /// See [nng_listener_create](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_create.3).
    pub(crate) fn create(socket: Arc<NngSocket>, url: &str) -> NngResult<Self> {
        unsafe {
            let mut listener = nng_listener { id: 0 };
            let (_cstring, url) = to_cstr(url)?;
            NngFail::succeed(
                nng_listener_create(&mut listener, socket.nng_socket(), url),
                NngListener { listener, socket },
            )
        }
    }

    /// See [nng_listener_start](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_start.3).
    pub fn start(&self) -> NngReturn {
        // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
        // be set before the dialer is started.
        unsafe { NngFail::from_i32(nng_listener_start(self.listener, 0)) }
    }
}

/// "Unsafe" version of `NngListener`.  Merely wraps `nng_listener` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
pub struct UnsafeListener {
    listener: nng_listener,
}

impl UnsafeListener {
    pub(crate) fn new(listener: nng_listener) -> Self {
        Self { listener }
    }

    /// See [nng_listener_id](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_listener_id(self.listener) }
    }
}
