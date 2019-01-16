//! Pair protocol

use super::*;
use runng_sys::*;

/// Half of pair pattern.  See [nng_pair](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair.7).
#[derive(Clone)]
pub struct Pair1 {
    socket: NngSocket,
}

impl Pair1 {
    /// Create a new pair socket.  See [nng_pair_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair_open.3).
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket: &mut nng_socket| unsafe { nng_pair1_open(socket) },
            |socket| Pair1 { socket },
        )
    }
}

impl Socket for Pair1 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Dial for Pair1 {}
impl Listen for Pair1 {}
impl SendMsg for Pair1 {}
impl RecvMsg for Pair1 {}

impl AsyncSocket for Pair1 {
    type ContextType = AsyncPairContext;
}
