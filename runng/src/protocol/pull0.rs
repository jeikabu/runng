use runng_sys::*;
use super::*;

pub struct Pull0 {
    socket: NngSocket
}

impl Pull0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_pull0_open(socket) }, 
            |socket| Pull0{ socket }
        )
    }
}

impl Socket for Pull0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Pull0 {}
impl Listen for Pull0 {}
impl RecvMsg for Pull0 {}

impl AsyncSocket for Pull0 {
    type ContextType = AsyncPullContext;
}
