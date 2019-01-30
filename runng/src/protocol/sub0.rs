//! Publisher/subscriber pattern.

use super::*;
use crate::asyncio::*;
use runng_sys::*;

/// Subscribe half of publisher/subscriber pattern.  See [nng_sub](https://nanomsg.github.io/nng/man/v1.1.0/nng_sub.7).
pub struct Sub0 {
    socket: NngSocket,
}

impl Sub0 {
    /// Create a sub socket.  See [nng_sub_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_sub_open.3).
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_sub0_open(socket) },
            |socket| Sub0 { socket },
        )
    }
}

impl Subscribe for Sub0 {
    /// Subscribe to a topic.
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe { subscribe(self.socket.nng_socket(), topic) }
    }
}

impl Socket for Sub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Dial for Sub0 {}
impl Listen for Sub0 {}
impl RecvMsg for Sub0 {}

impl AsyncStream for Sub0 {
    type ContextType = SubscribeAsyncHandle;
}
