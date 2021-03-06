//! Push/pull ("pipeline") pattern.

use super::*;
use crate::asyncio::*;
use runng_sys::*;

/// Push half of push/pull ("pipeline") pattern.  See [nng_push](https://nng.nanomsg.org/man/v1.2.2/nng_push.7).
#[derive(Clone, Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_socket_"]
pub struct Push0 {
    socket: NngSocket,
}

impl Push0 {
    /// Create a push socket.  See [nng_push_open](https://nng.nanomsg.org/man/v1.2.2/nng_push_open.3).
    pub fn open() -> Result<Self> {
        nng_open(
            |socket| unsafe { nng_push0_open(socket) },
            |socket| Push0 { socket },
        )
    }
}

impl GetSocket for Push0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Socket for Push0 {}
impl Dial for Push0 {}
impl Listen for Push0 {}
impl SendSocket for Push0 {}

impl AsyncSocket for Push0 {
    type ContextType = PushAsyncHandle;
}
