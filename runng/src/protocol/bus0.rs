//! Bus protocol.

use super::*;
use crate::asyncio::*;
use runng_sys::*;

/// Half of pair pattern.  See [nng_pair](https://nng.nanomsg.org/man/v1.2.2/nng_pair.7).
#[derive(Clone, Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_socket_"]
pub struct Bus0 {
    socket: NngSocket,
}

impl Bus0 {
    /// Create a new pair socket.  See [nng_pair_open](https://nng.nanomsg.org/man/v1.2.2/nng_pair_open.3).
    pub fn open() -> Result<Self> {
        let socket_create_func = |socket| Self { socket };
        nng_open(
            |socket: &mut nng_socket| unsafe { nng_bus0_open(socket) },
            socket_create_func,
        )
    }
}

impl GetSocket for Bus0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Socket for Bus0 {}
impl Dial for Bus0 {}
impl Listen for Bus0 {}
impl SendSocket for Bus0 {}
impl RecvSocket for Bus0 {}

impl AsyncSocket for Bus0 {
    type ContextType = BusAsyncHandle;
}
