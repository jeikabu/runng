use runng_sys::*;
use super::*;

pub struct Rep0 {
    socket: NngSocket
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        nng_open(|socket| unsafe { nng_rep0_open(socket) }, 
            |socket| Rep0{ socket }
        )
    }
}

impl Socket for Rep0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Listen for Rep0 {}
impl RecvMsg for Rep0 {}

impl AsyncSocket for Rep0 {
    type ContextType = AsyncReplyContext;
}
