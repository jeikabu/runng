//! Push/pull ("pipeline") pattern.

use super::*;
use runng_sys::*;
use std::sync::Arc;

/// Pull half of push/pull ("pipeline") pattern.  See [nng_pull](https://nanomsg.github.io/nng/man/v1.1.0/nng_pull.7).
pub struct Pull0 {
    socket: Arc<NngSocket>,
}

impl Pull0 {
    /// Create a pull socket.  See [nng_pull_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_pull_open.3).
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_pull0_open(socket) },
            |socket| Pull0 { socket },
        )
    }
}

impl Socket for Pull0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn clone_socket(&self) -> Arc<NngSocket> {
        self.socket.clone()
    }
}

impl Dial for Pull0 {}
impl Listen for Pull0 {}
impl RecvMsg for Pull0 {}

impl AsyncSocket for Pull0 {
    type ContextType = AsyncPullContext;
}
