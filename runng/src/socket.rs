//! Socket basics

use runng_sys::*;
use std::ffi::CString;
use super::*;

/// Wraps `nng_socket`.  See [nng_socket](https://nanomsg.github.io/nng/man/v1.1.0/nng_socket.5).
pub struct NngSocket {
    socket: nng_socket,
}

impl NngSocket {
    pub fn new(socket: nng_socket) -> NngSocket {
        NngSocket { socket }
    }
    /// Obtain underlying `nng_socket`
    pub unsafe fn nng_socket(&self) -> nng_socket {
        self.socket
    }
}

impl Drop for NngSocket {
    fn drop(&mut self) {
        unsafe {
            debug!("Socket close: {:?}", self.socket);
            let res = NngFail::from_i32(nng_close(self.socket));
            match res {
                Ok(()) => {},
                // Can't panic here.  Thrift's TIoChannel::split() clones the socket handle so we may get ECLOSED
                Err(NngFail::Err(NngError::ECLOSED)) => {},
                Err(res) => {
                    debug!("nng_close {:?}", res);
                    panic!(res);
                },
            }
        }
    }
}

impl Socket for NngSocket {
    fn socket(&self) -> &NngSocket {
        self
    }
    fn take(self) -> NngSocket {
        self
    }
}

impl SendMsg for NngSocket {}
impl RecvMsg for NngSocket {}

pub trait Socket: Sized {
    fn socket(&self) -> &NngSocket;
    fn take(self) -> NngSocket;
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

/// `Socket` that can accept connections ("listen") from other `Socket`s.
pub trait Listen: Socket {
    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen(self, url: &str) -> NngResult<Self> {
        unsafe {
            let res = nng_listen(self.nng_socket(), to_cstr(url).1, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }
}

/// `Socket` that can connect to ("dial") another `Socket`.
pub trait Dial: Socket {
    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial(self, url: &str) -> NngResult<Self> {
        unsafe {
            let res = nng_dial(self.nng_socket(), to_cstr(url).1, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(string: &str) -> (CString, *const i8) {
    let string = CString::new(string).unwrap();
    let ptr = string.as_bytes_with_nul().as_ptr() as *const i8;
    (string, ptr)
}

/// `Socket` that can send messages.
pub trait SendMsg: Socket {
    /// Send a message.  See [nng_sendmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_sendmsg.3).
    fn send(&self, msg: msg::NngMsg) -> NngReturn {
        let res = unsafe {
            nng_sendmsg(self.nng_socket(), msg.take(), 0)
        };
        NngFail::from_i32(res)
    }
}

/// `Socket` that can receive messages.
pub trait RecvMsg: Socket {
    /// Receive a message.  See [nng_recvmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_recvmsg.3).
    fn recv(&self) -> NngResult<msg::NngMsg> {
        unsafe {
            let mut recv_ptr: *mut nng_msg = std::ptr::null_mut();
            let res = nng_recvmsg(self.nng_socket(), &mut recv_ptr, 0);
            NngFail::succeed_then(res, || msg::NngMsg::new_msg(recv_ptr))
        }
    }
}
