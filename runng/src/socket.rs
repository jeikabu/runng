//! Socket basics

use runng_sys::*;
use std::ffi::CString;
use super::{
    *,
    dialer::NngDialer,
};

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
            let (_, ptr) = to_cstr(url)?;
            let res = nng_listen(self.nng_socket(), ptr, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }
}

/// `Socket` that can connect to ("dial") another `Socket`.
pub trait Dial: Socket {
    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial(self, url: &str) -> NngResult<Self> {
        unsafe {
            let (_, ptr) = to_cstr(url)?;
            let res = nng_dial(self.nng_socket(), ptr, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }

    fn dialer_create(self, url: &str) -> NngResult<NngDialer> {
        unsafe {
            NngDialer::new(self.nng_socket(), url)
        }
    }
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

/// See [nng_getopt](https://nanomsg.github.io/nng/man/v1.1.0/nng_getopt.3) and [nng_setopt](https://nanomsg.github.io/nng/man/v1.1.0/nng_setopt.3)
impl Opts for NngSocket {
    fn getopt_bool(&self, option: NngOption) -> NngResult<bool> {
        unsafe {
            let mut value: bool = Default::default();
            NngFail::succeed(nng_getopt_bool(self.socket, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_int(&self, option: NngOption) -> NngResult<i32> {
        unsafe {
            let mut value: i32 = Default::default();
            NngFail::succeed(nng_getopt_int(self.socket, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_size(&self, option: NngOption) -> NngResult<usize> {
        unsafe {
            let mut value: usize = Default::default();
            NngFail::succeed(nng_getopt_size(self.socket, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_uint64(&self, option: NngOption) -> NngResult<u64> {
        unsafe {
            let mut value: u64 = Default::default();
            NngFail::succeed(nng_getopt_uint64(self.socket, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_string(&self, option: NngOption) -> NngResult<NngString> {
        unsafe {
            let mut value: *mut ::std::os::raw::c_char = std::ptr::null_mut();
            let res = nng_getopt_string(self.socket, option.as_cptr(), &mut value);
            NngFail::from_i32(res)?;
            Ok(NngString::new(value))
        }
    }

    fn setopt_bool(&mut self, option: NngOption, value: bool) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_setopt_bool(self.socket, option.as_cptr(), value))
        }
    }
    fn setopt_int(&mut self, option: NngOption, value: i32) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_setopt_int(self.socket, option.as_cptr(), value))
        }
    }
    fn setopt_size(&mut self, option: NngOption, value: usize) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_setopt_size(self.socket, option.as_cptr(), value))
        }
    }
    fn setopt_uint64(&mut self, option: NngOption, value: u64) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_setopt_uint64(self.socket, option.as_cptr(), value))
        }
    }
    fn setopt_string(&mut self, option: NngOption, value: &str) -> NngReturn {
        unsafe {
            let (_, value) = to_cstr(value)?;
            NngFail::from_i32(nng_setopt_string(self.socket, option.as_cptr(), value))
        }
    }
}