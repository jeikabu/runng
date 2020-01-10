//! Request/reply pattern.

use super::*;
use crate::{asyncio::*, *};
use runng_sys::*;

/// Request half of request/reply pattern.  See [nng_req](https://nng.nanomsg.org/man/v1.2.2/nng_req.7).
#[derive(Clone, Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_socket_"]
pub struct Req0 {
    socket: NngSocket,
}

impl Req0 {
    /// Create a new request socket.  See [nng_req_open](https://nng.nanomsg.org/man/v1.2.2/nng_req_open.3).
    pub fn open() -> Result<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0 { socket };
        nng_open(open_func, socket_create_func)
    }
}

impl GetSocket for Req0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Socket for Req0 {}
impl Dial for Req0 {}
impl Listen for Req0 {}
impl SendSocket for Req0 {}
impl RecvSocket for Req0 {}

impl AsyncSocket for Req0 {
    type ContextType = RequestAsyncHandle;
}
