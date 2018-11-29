use runng_sys::*;
use super::*;

pub struct Pub0 {
    socket: NngSocket
}

impl Pub0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_pub0_open(socket) };
        let socket_create_func = |socket| Pub0{ socket };
        nng_open(open_func, socket_create_func)
    }
}

impl Socket for Pub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Pub0 {}
impl Listen for Pub0 {}
impl SendMsg for Pub0 {}

impl AsyncSocket for Pub0 {
    type ContextType = AsyncPublishContext;
}