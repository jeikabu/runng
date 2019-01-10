//! Socket basics

use super::{dialer::NngDialer, listener::NngListener, *};
use runng_derive::NngGetOpts;
use runng_sys::*;
use std::sync::Arc;

/// Wraps `nng_socket`.  See [nng_socket](https://nanomsg.github.io/nng/man/v1.1.0/nng_socket.5).
#[derive(NngGetOpts)]
#[prefix = "nng_"]
pub struct NngSocket {
    #[nng_member]
    socket: nng_socket,
}

impl NngSocket {
    /// Create a new `NngSocket`.
    pub fn create(socket: nng_socket) -> Arc<Self> {
        Arc::new(NngSocket { socket })
    }

    /// Obtain underlying `nng_socket`
    pub unsafe fn nng_socket(&self) -> nng_socket {
        self.socket
    }

    /// Register pipe notification callback.  See [nng_pipe_notify](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_notify.3).
    #[cfg(feature = "pipes")]
    pub fn notify(
        &self,
        event: pipe::PipeEvent,
        callback: pipe::PipeNotifyCallback,
        argument: pipe::PipeNotifyCallbackArg,
    ) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_pipe_notify(
                self.socket,
                event as i32,
                Some(callback),
                argument,
            ))
        }
    }
}

impl Drop for NngSocket {
    fn drop(&mut self) {
        unsafe {
            debug!("Socket close: {:?}", self.socket);
            let res = NngFail::from_i32(nng_close(self.socket));
            match res {
                Ok(()) => {}
                // Can't panic here.  Thrift's TIoChannel::split() clones the socket handle so we may get ECLOSED
                Err(NngFail::Err(NngError::ECLOSED)) => {}
                Err(res) => {
                    debug!("nng_close {:?}", res);
                    panic!("nng_close {:?}", res);
                }
            }
        }
    }
}

impl Socket for NngSocket {
    fn socket(&self) -> &NngSocket {
        self
    }
    fn clone_socket(&self) -> Arc<NngSocket> {
        panic!()
    }
}

impl SendMsg for NngSocket {}
impl RecvMsg for NngSocket {}

/// Type which exposes a `NngSocket`.
pub trait Socket: Sized {
    // Obtain underlying `NngSocket`.
    fn socket(&self) -> &NngSocket;
    fn clone_socket(&self) -> Arc<NngSocket>;
    /// Obtain underlying `nng_socket`.
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

/// `Socket` that can accept connections from ("listen" to) other `Socket`s.
pub trait Listen: Socket {
    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen(self, url: &str) -> NngResult<Self> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_listen(self.nng_socket(), ptr, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }

    fn listener_create(&self, url: &str) -> NngResult<NngListener> {
        NngListener::create(self.clone_socket(), url)
    }
}

/// `Socket` that can connect to ("dial") another `Socket`.
pub trait Dial: Socket {
    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial(self, url: &str) -> NngResult<Self> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_dial(self.nng_socket(), ptr, std::ptr::null_mut(), 0);
            NngFail::succeed(res, self)
        }
    }

    fn dialer_create(&self, url: &str) -> NngResult<NngDialer> {
        NngDialer::create(self.clone_socket(), url)
    }
}

/// `Socket` that can send messages.
pub trait SendMsg: Socket {
    /// Send a message.  See [nng_sendmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_sendmsg.3).
    fn send(&self, msg: msg::NngMsg) -> NngReturn {
        let res = unsafe { nng_sendmsg(self.nng_socket(), msg.take(), 0) };
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

/// "Unsafe" version of `NngSocket`.  Merely wraps `nng_socket` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
pub struct UnsafeSocket {
    socket: nng_socket,
}

impl UnsafeSocket {
    pub fn new(socket: nng_socket) -> Self {
        Self { socket }
    }

    /// See [nng_socket_id](https://nanomsg.github.io/nng/man/v1.1.0/nng_socket_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_socket_id(self.socket) }
    }
}
