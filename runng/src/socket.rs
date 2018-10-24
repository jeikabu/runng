use runng_sys::*;
use std::ffi::CString;
use super::*;

pub struct NngSocket {
    socket: nng_socket,
}

impl NngSocket {
    pub fn new(socket: nng_socket) -> NngSocket {
        NngSocket { socket }
    }
    pub unsafe fn nng_socket(&self) -> nng_socket {
        self.socket
    }
}

impl Drop for NngSocket {
    fn drop(&mut self) {
        unsafe {
            println!("Socket close: {:?}", self.socket);
            let res = NngFail::from_i32(nng_close(self.socket));
            if let Err(res) = res {
                println!("nng_close {:?}", res);
                panic!(res);
            }
        }
    }
}

impl Socket for NngSocket {
    fn socket(&self) -> &NngSocket {
        self
    }
}

impl SendMsg for NngSocket {}
impl RecvMsg for NngSocket {}

pub trait Socket {
    fn socket(&self) -> &NngSocket;
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

pub trait Listen: Socket {
    fn listen(&self, url: &str) -> NngReturn {
        unsafe {
            let res = nng_listen(self.nng_socket(), to_cstr(url).1, std::ptr::null_mut(), 0);
            NngFail::from_i32(res)
        }
    }
}

pub trait Dial: Socket {
    fn dial(&self, url: &str) -> NngReturn {
        unsafe {
            let res = nng_dial(self.nng_socket(), to_cstr(url).1, std::ptr::null_mut(), 0);
            NngFail::from_i32(res)
        }
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(string: &str) -> (CString, *const i8) {
    let string = CString::new(string).unwrap();
    let ptr = string.as_bytes_with_nul().as_ptr() as *const i8;
    (string, ptr)
}

pub trait SendMsg: Socket {
    fn send(&self, msg: msg::NngMsg) -> NngReturn {
        let mut req_msg: *mut nng_msg = std::ptr::null_mut();
        let res = unsafe {
            nng_sendmsg(self.nng_socket(), msg.take(), 0)
        };
        NngFail::from_i32(res)
    }
}

pub trait RecvMsg: Socket {
    fn recv(&self) -> NngResult<msg::NngMsg> {
        unsafe {
            let mut recv_ptr: *mut nng_msg = std::ptr::null_mut();
            let res = nng_recvmsg(self.nng_socket(), &mut recv_ptr, 0);
            NngFail::succeed_then(res, || msg::NngMsg::new_msg(recv_ptr))
        }
    }
}
