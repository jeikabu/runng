//! Request/reply pattern.

use super::*;
use runng_sys::*;
use std::sync::Arc;

/// Reply half of request/reply pattern.  See [nng_rep](https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7).
pub struct Rep0 {
    socket: Arc<NngSocket>,
}

impl Rep0 {
    /// Create a new reply socket.  See [nng_rep_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_rep_open.3).
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_rep0_open(socket) },
            |socket| Rep0 { socket },
        )
    }
}

impl Socket for Rep0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn clone_socket(&self) -> Arc<NngSocket> {
        self.socket.clone()
    }
}

impl Listen for Rep0 {}
impl Dial for Rep0 {}
impl RecvMsg for Rep0 {}
impl SendMsg for Rep0 {}

impl AsyncSocket for Rep0 {
    type ContextType = AsyncReplyContext;
}
