
use runng_sys::*;
use super::*;

pub struct Req0 {
    socket: Socket
}

pub struct Rep0 {
    socket: Socket
}

impl Req0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = Socket::new();
        let res = unsafe { nng_req0_open(&mut socket.socket) };
        if res == 0 {
            Ok(Req0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = Socket::new();
        let res = unsafe { nng_rep0_open(&mut socket.socket) };
        if res == 0 {
            Ok(Rep0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}
