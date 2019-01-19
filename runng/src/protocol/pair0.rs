//! Pair protocol

use super::*;
use runng_sys::*;

/// Half of pair pattern.  See [nng_pair](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair.7).
pub struct Pair0 {
    socket: NngSocket,
}

impl Pair0 {
    /// Create a new pair socket.  See [nng_pair_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair_open.3).
    pub fn open() -> NngResult<Self> {
        let socket_create_func = |socket| Pair0 { socket };
        nng_open(
            |socket: &mut nng_socket| unsafe { nng_pair0_open(socket) },
            socket_create_func,
        )
    }
}

impl Socket for Pair0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Dial for Pair0 {}
impl Listen for Pair0 {}
impl SendMsg for Pair0 {}
impl RecvMsg for Pair0 {}

impl AsyncSocket for Pair0 {
    type ContextType = AsyncPairContext;
}
