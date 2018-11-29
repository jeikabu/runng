//! Push/pull ("pipeline") pattern.

use runng_sys::*;
use super::*;

/// Push half of push/pull ("pipeline") pattern.  See [nng_push](https://nanomsg.github.io/nng/man/v1.1.0/nng_push.7).
pub struct Push0 {
    socket: Arc<NngSocket>
}

impl Push0 {
    /// Create a push socket.  See [nng_push_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_push_open.3).
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_push0_open(socket) }, 
            |socket| Push0{ socket }
        )
    }
}

impl Socket for Push0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn clone_socket(&self) -> Arc<NngSocket> {
        self.socket.clone()
    }
}

impl Dial for Push0 {}
impl Listen for Push0 {}
impl SendMsg for Push0 {}

impl AsyncSocket for Push0 {
    type ContextType = AsyncPublishContext;
}
