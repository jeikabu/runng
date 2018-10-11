use runng_sys::*;
use super::*;
use std::ffi::CString;

pub trait Socket {
    fn socket(&self) -> nng_socket;
}

pub struct NngSocket {
    socket: nng_socket,
}

impl NngSocket {
    pub fn new(socket: nng_socket) -> NngSocket {
        NngSocket { socket }
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
            println!("Socket close: {:?}", self.socket);
            let res = nng_close(self.socket);
            if res != 0 {
                println!("nng_close {:?}", NngFail::from_i32(res));
                panic!(res);
            }
        }
    }
}


pub trait Listen: Socket {
    fn listen(&self, url: &str) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_listen(self.socket(), to_cstr(url).1, std::ptr::null_mut(), 0))
            }
    }
}

pub trait Dial: Socket {
    fn dial(&self, url: &str) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dial(self.socket(), to_cstr(url).1, std::ptr::null_mut(), 0))
        }
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(string: &str) -> (CString, *const i8) {
    let url = CString::new(string).unwrap();
    let ptr = url.as_bytes_with_nul().as_ptr() as *const i8;
    (url, ptr)
}

pub trait SendMsg: Socket {
    fn send(&self) -> NngReturn {
        let mut req_msg: *mut nng_msg = std::ptr::null_mut();
        let res = unsafe {
            let res = nng_msg_alloc(&mut req_msg, 0);
            if res != 0 {
                res
            } else {
                nng_sendmsg(self.socket(), req_msg, 0)
            }
        };
        NngFail::from_i32(res)
    }
}

pub trait RecvMsg: Socket {
    fn recv(&self) -> NngResult<nng_msg> {
        unsafe {
            let mut recv_ptr: *mut nng_msg = std::ptr::null_mut();
            let res = nng_recvmsg(self.socket(), &mut recv_ptr, 0);
            NngFail::succeed_then(res, || *recv_ptr)
        }
    }
}
