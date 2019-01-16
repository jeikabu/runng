//! Publisher/subscriber pattern.

use super::*;
use runng_sys::*;

/// Publish half of publisher/subscriber pattern.  See [nng_pub](https://nanomsg.github.io/nng/man/v1.1.0/nng_pub.7).
pub struct Pub0 {
    socket: NngSocket,
}

impl Pub0 {
    /// Create a pub socket.  See [nng_pub_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_pub_open.3).
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_pub0_open(socket) };
        let socket_create_func = |socket| Pub0 { socket };
        nng_open(open_func, socket_create_func)
    }
}

impl Socket for Pub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Dial for Pub0 {}
impl Listen for Pub0 {}
impl SendMsg for Pub0 {}

impl AsyncSocket for Pub0 {
    type ContextType = AsyncPushContext;
}
