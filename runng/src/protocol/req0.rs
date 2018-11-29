use runng_sys::*;
use super::*;

pub struct Req0 {
    socket: NngSocket
}

impl Req0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0{ socket };
        nng_open(open_func, socket_create_func)
    }
}

impl Socket for Req0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Req0 {}
impl SendMsg for Req0 {}

impl AsyncSocket for Req0 {
    type ContextType = AsyncRequestContext;
}
