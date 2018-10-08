use super::*;
use std::ffi::CString;

pub trait Socket {
    fn socket(&self) -> nng_socket;
}

pub struct NngSocket {
    socket: nng_socket,
}

impl NngSocket {
    pub fn new() -> NngSocket {
        NngSocket { socket: nng_socket { id: 0 } }
    }
}

impl Socket for NngSocket {
    fn socket(&self) -> nng_socket {
        self.socket
    }
}

impl Drop for NngSocket {
    fn drop(&mut self) {
        unsafe {
            let res = nng_close(self.socket);
            if res != 0 {
                println!("nng_close {:?}", NngFail::from_i32(res));
                panic!(res);
            }
        }
    }
}


pub trait Listen: Socket {
    fn listen(&self, url: &str) -> NngResult<()> {
        let res = unsafe {
            nng_listen(self.socket(), to_cstr(url).1, std::ptr::null_mut(), 0)
            };
        NngReturn::from(res, ())
    }
}

pub trait Dial: Socket {
    fn dial(&self, url: &str) -> NngResult<()> {
        let res = unsafe {
            nng_dial(self.socket(), to_cstr(url).1, std::ptr::null_mut(), 0)
        };
        NngReturn::from(res, ())
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(string: &str) -> (CString, *const i8) {
    let url = CString::new(string).unwrap();
    let ptr = url.as_bytes_with_nul().as_ptr() as *const i8;
    (url, ptr)
}

pub trait Send: Socket {
    fn send(&self) -> NngResult<()> {
        let mut req_msg = nng_msg::new();
        let mut req_msg = &mut req_msg as *mut nng_msg;
        let res = unsafe {
            let res = nng_msg_alloc(&mut req_msg, 0);
            if res != 0 {
                res
            } else {
                nng_sendmsg(self.socket(), req_msg, 0)
            }
        };
        NngReturn::from(res, ())
    }
}

pub trait Recv: Socket {
    fn recv(&self) -> NngResult<nng_msg> {
        let mut recv_msg = nng_msg::new();
        let mut recv_ptr = &mut recv_msg as *mut nng_msg;
        let res = unsafe {
            nng_recvmsg(self.socket(), &mut recv_ptr, 0)
        };
        NngReturn::from(res, recv_msg)
    }
}
