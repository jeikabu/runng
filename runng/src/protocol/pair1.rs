//! Pair protocol.

use super::*;
use crate::asyncio::*;
use runng_sys::*;

/// Half of pair pattern.  See [nng_pair](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair.7).
#[derive(Clone, Debug, NngSetOpts)]
#[prefix = "nng_socket_"]
pub struct Pair1 {
    socket: NngSocket,
}

impl Pair1 {
    /// Create a new pair socket.  See [nng_pair_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_pair_open.3).
    pub fn open() -> Result<Self> {
        nng_open(
            |socket: &mut nng_socket| unsafe { nng_pair1_open(socket) },
            |socket| Pair1 { socket },
        )
    }
}

impl GetSocket for Pair1 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Socket for Pair1 {}
impl Dial for Pair1 {}
impl Listen for Pair1 {}
impl SendSocket for Pair1 {}
impl RecvSocket for Pair1 {}

impl AsyncSocket for Pair1 {
    type ContextType = PairAsyncHandle;
}

impl AsyncStream for Pair1 {
    type ContextType = PairStreamHandle;
}
