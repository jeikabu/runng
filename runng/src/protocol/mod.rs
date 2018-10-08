pub mod reqrep;

pub use self::reqrep::*;

use runng_sys::*;
use super::*;

pub struct Req0 {
    socket: NngSocket
}

pub struct Rep0 {
    socket: NngSocket
}

impl Req0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = NngSocket::new();
        let res = unsafe { nng_req0_open(&mut socket.socket()) };
        if res == 0 {
            Ok(Req0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = NngSocket::new();
        let res = unsafe { nng_rep0_open(&mut socket.socket()) };
        if res == 0 {
            Ok(Rep0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}