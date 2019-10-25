//! Listeners accept connections from dialers.

use super::*;
use runng_derive::{NngGetOpts, NngSetOpts};
use runng_sys::*;

/// Wraps `nng_listener`.  See [nng_listener](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener.5).
#[derive(Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_listener_"]
pub struct NngListener {
    listener: nng_listener,
    socket: NngSocket,
}

impl NngListener {
    /// See [nng_listener_create](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_create.3).
    pub(crate) fn new(socket: NngSocket, url: &str) -> Result<Self> {
        unsafe {
            let mut listener = nng_listener::default();
            let (_cstring, url) = to_cstr(url)?;
            let res = nng_listener_create(&mut listener, socket.nng_socket(), url);
            Error::zero_map(res, || NngListener { listener, socket })
        }
    }

    /// See [nng_listener_start](https://nanomsg.github.io/nng/man/v1.1.0/nng_listener_start.3).
    pub fn start(&self) -> Result<()> {
        // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
        // be set before the dialer is started.
        unsafe { nng_int_to_result(nng_listener_start(self.listener, 0)) }
    }
}

impl NngWrapper for NngListener {
    type NngType = nng_listener;
    unsafe fn get_nng_type(&self) -> Self::NngType {
        self.listener
    }
}

impl GetSocket for NngListener {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

/// "Unsafe" version of `NngListener`.  Merely wraps `nng_listener` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
#[derive(Debug)]
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
