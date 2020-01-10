//! Pair protocol.

use super::*;
use crate::asyncio::*;
use runng_sys::*;

/// Half of pair pattern.  See [nng_pair](https://nng.nanomsg.org/man/v1.2.2/nng_pair.7).
#[derive(Clone, Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_socket_"]
pub struct Pair0 {
    socket: NngSocket,
}

impl Pair0 {
    /// Create a new pair socket.  See [nng_pair_open](https://nng.nanomsg.org/man/v1.2.2/nng_pair_open.3).
    pub fn open() -> Result<Self> {
        let socket_create_func = |socket| Pair0 { socket };
        nng_open(
            |socket: &mut nng_socket| unsafe { nng_pair0_open(socket) },
            socket_create_func,
        )
    }
}

impl GetSocket for Pair0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Socket for Pair0 {}
impl Dial for Pair0 {}
impl Listen for Pair0 {}
impl SendSocket for Pair0 {}
impl RecvSocket for Pair0 {}

impl AsyncSocket for Pair0 {
    type ContextType = PairAsyncHandle;
}

impl AsyncStream for Pair0 {
    type ContextType = PairStreamHandle;
}
