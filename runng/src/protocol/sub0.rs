use runng_sys::*;
use super::*;

pub struct Sub0 {
    socket: NngSocket
}

impl Sub0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_sub0_open(socket) }, 
            |socket| Sub0{ socket }
        )
    }
}

impl Subscribe for Sub0 {
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe {
            subscribe(self.socket.nng_socket(), topic)
        }
    }
}

impl Socket for Sub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Sub0 {}
impl RecvMsg for Sub0 {}

impl AsyncSocket for Sub0 {
    type ContextType = AsyncSubscribeContext;
}
