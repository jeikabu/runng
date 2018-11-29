use runng_sys::*;
use super::*;

pub struct Push0 {
    socket: NngSocket
}

impl Push0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_push0_open(socket) }, 
            |socket| Push0{ socket }
        )
    }
}

impl Socket for Push0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Push0 {}
impl Listen for Push0 {}
impl SendMsg for Push0 {}

impl AsyncSocket for Push0 {
    type ContextType = AsyncPublishContext;
}
