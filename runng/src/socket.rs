//! Socket basics
//!
//! Instantiating any of the various "protocols" creates a socket.
//! A socket may be cloned and it will increase the reference count of the underlying `nng_socket`.
//! Depending on the gurantees of the originating protocol, simultaneous use of the socket __may not be safe__.
//! When the last reference to the socket is dropped, `nng_close()` will be called.

use crate::{dialer::NngDialer, listener::NngListener, *};
use bitflags::bitflags;
use runng_sys::*;
use std::{fmt, result, sync::Arc};

bitflags! {
    /// Flags used with [`SendSocket`](trait.SendSocket.html) and [`RecvSocket`](trait.RecvSocket.html).
    #[derive(Default)]
    pub struct Flags: i32 {
        const NONBLOCK = NNG_FLAG_NONBLOCK as i32;
        const ALLOC = NNG_FLAG_ALLOC as i32;
    }
}

bitflags! {
    /// Flags used with [`Listen`](trait.Listen.html) and [`Dial`](trait.Dial.html).
    #[derive(Default)]
    pub struct SocketFlags: i32 {
        const NONBLOCK = NNG_FLAG_NONBLOCK as i32;
    }
}

/// Wraps `nng_socket`.  See [nng_socket](https://nanomsg.github.io/nng/man/v1.1.0/nng_socket.5).
#[derive(Debug)]
pub struct NngSocket {
    socket: Arc<InnerSocket>,
}

impl NngSocket {
    /// Create a new `NngSocket`.
    pub fn new(socket: nng_socket) -> Self {
        let socket = Arc::new(InnerSocket { socket });
        NngSocket { socket }
    }

    /// Obtain underlying `nng_socket`
    pub unsafe fn nng_socket(&self) -> nng_socket {
        self.socket.socket
    }

    /// Register pipe notification callback.  See [nng_pipe_notify](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_notify.3).
    #[cfg(feature = "pipes")]
    pub fn notify(
        &self,
        event: nng_pipe_ev,
        callback: pipe::PipeNotifyCallback,
        argument: pipe::PipeNotifyCallbackArg,
    ) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_pipe_notify(
                self.nng_socket(),
                event,
                Some(callback),
                argument,
            ))
        }
    }
}

impl<T> NngWrapper for T
where
    T: Socket,
{
    type NngType = nng_socket;
    unsafe fn get_nng_type(&self) -> Self::NngType {
        self.socket().nng_socket()
    }
}

impl GetSocket for NngSocket {
    fn socket(&self) -> &NngSocket {
        self
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        self
    }
}

impl Socket for NngSocket {}
impl SendSocket for NngSocket {}
impl RecvSocket for NngSocket {}

impl Clone for NngSocket {
    fn clone(&self) -> Self {
        let socket = self.socket.clone();
        Self { socket }
    }
}

/// Type which exposes an [`NngSocket`](struct.NngSocket.html).
pub trait GetSocket {
    /// Obtain underlying `NngSocket`.
    fn socket(&self) -> &NngSocket;
    fn socket_mut(&mut self) -> &mut NngSocket;

    /// Obtain underlying `nng_socket`.
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

/// Type which __is__ an [`NngSocket`](struct.NngSocket.html).
pub trait Socket: GetSocket + Sized {
    /// Helper to chain constructors with methods that return `&Self`.
    ///
    /// # Examples
    /// ```
    /// use runng::{Listen, protocol::Pair0, Socket};
    /// fn main() -> runng::Result<()> {
    ///     let mut socket0 = Pair0::open()?;
    ///     socket0.listen("inproc://socket0")?;
    ///     // VS
    ///     let socket1 = Pair0::open()?.with(|sock| sock.listen("inproc://socket1"))?;
    ///     Ok(())
    /// }
    /// ```
    fn with<T>(mut self, setup: T) -> Result<Self>
    where
        T: FnOnce(&mut Self) -> Result<&mut Self>,
    {
        setup(&mut self)?;
        Ok(self)
    }
}

/// `Socket` that can accept connections from ("listen" to) other `Socket`s.
pub trait Listen: Socket {
    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen(&mut self, url: &str) -> Result<&mut Self> {
        self.listen_flags(url, Default::default())
    }

    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen_flags(&mut self, url: &str, flags: SocketFlags) -> Result<&mut Self> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_listen(self.nng_socket(), ptr, std::ptr::null_mut(), flags.bits());
            Error::zero_map(res, || self)
        }
    }

    fn listener_create(&self, url: &str) -> Result<NngListener> {
        NngListener::new(self.socket().clone(), url)
    }
}

/// `Socket` that can connect to ("dial") another `Socket`.
pub trait Dial: Socket {
    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial(&mut self, url: &str) -> Result<&mut Self> {
        self.dial_flags(url, Default::default())
    }

    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial_flags(&mut self, url: &str, flags: SocketFlags) -> Result<&mut Self> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_dial(self.nng_socket(), ptr, std::ptr::null_mut(), flags.bits());
            Error::zero_map(res, || self)
        }
    }

    fn dialer_create(&self, url: &str) -> Result<NngDialer> {
        NngDialer::new(self.socket().clone(), url)
    }
}

#[derive(Debug)]
pub struct SendError<T: fmt::Debug> {
    pub error: Error,
    pub message: T,
}

impl<T: fmt::Debug> SendError<T> {
    pub fn into_inner(self) -> T {
        self.message
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

impl<T: fmt::Debug> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.error, self.message)
    }
}

/// `Socket` that can send data.
pub trait SendSocket: Socket {
    /// Send data.  See [nng_send](https://nanomsg.github.io/nng/man/v1.1.0/nng_send.3).
    fn send(&self, data: &[u8]) -> Result<()> {
        self.send_flags(data, Default::default())
    }

    /// Send data with [`Flags`](struct.Flags.html).  See [nng_send](https://nanomsg.github.io/nng/man/v1.1.0/nng_send.3).
    fn send_flags(&self, data: &[u8], flags: Flags) -> Result<()> {
        unsafe {
            let ptr = data.as_ptr() as *mut std::os::raw::c_void;
            let res = nng_send(self.nng_socket(), ptr, data.len(), flags.bits());
            nng_int_to_result(res)
        }
    }

    /// Sends data in "zero-copy" mode.  See `NNG_FLAG_ALLOC`.
    fn send_zerocopy(&self, data: mem::Alloc) -> result::Result<(), SendError<mem::Alloc>> {
        self.send_zerocopy_flags(data, Flags::ALLOC)
    }

    /// Send data in "zero-copy" mode with [`Flags`](struct.Flags.html).  See `NNG_FLAG_ALLOC`.
    fn send_zerocopy_flags(
        &self,
        data: mem::Alloc,
        flags: Flags,
    ) -> result::Result<(), SendError<mem::Alloc>> {
        let flags = (flags | Flags::ALLOC).bits();
        unsafe {
            let (ptr, size) = data.take();
            let res = nng_send(self.nng_socket(), ptr, size, flags);
            let error = nng_int_to_result(res);
            error.map_err(|error| {
                let message = mem::Alloc::from_raw_parts(ptr, size);
                SendError { error, message }
            })
        }
    }

    /// Send a [`NngMsg`](../msg/struct.NngMsg.html).  See [nng_sendmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_sendmsg.3).
    fn sendmsg(&self, msg: msg::NngMsg) -> Result<()> {
        let res = self.sendmsg_flags(msg, Default::default());
        res.map_err(|err| err.error)
    }

    /// Send a [`NngMsg`](../msg/struct.NngMsg.html) with [`Flags`](struct.Flags.html).  See [nng_sendmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_sendmsg.3).
    fn sendmsg_flags(
        &self,
        msg: msg::NngMsg,
        flags: Flags,
    ) -> result::Result<(), SendError<msg::NngMsg>> {
        unsafe {
            let ptr = msg.take();
            assert!(!ptr.is_null());
            let res = nng_sendmsg(self.nng_socket(), ptr, flags.bits());
            let error = nng_int_to_result(res);
            error.map_err(|error| {
                let message = msg::NngMsg::from_raw(ptr);
                SendError { error, message }
            })
        }
    }
}

/// `Socket` that can receive data.
pub trait RecvSocket: Socket {
    /// Receive data.  See [nng_recv](https://nanomsg.github.io/nng/man/v1.1.0/nng_recv.3).
    /// Lifetime of return value is same as input buffer.
    fn recv<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8]> {
        self.recv_flags(buffer, Default::default())
    }

    /// Receive data with [`Flags`](struct.Flags.html).
    /// Lifetime of return value is same as input buffer.
    fn recv_flags<'a>(&self, buffer: &'a mut [u8], flags: Flags) -> Result<&'a [u8]> {
        unsafe {
            let ptr = buffer.as_mut_ptr() as *mut core::ffi::c_void;
            let mut size = buffer.len();
            let res = nng_recv(self.nng_socket(), ptr, &mut size, flags.bits());
            let res = nng_int_to_result(res);
            res.map(|_| std::slice::from_raw_parts(buffer.as_ptr(), size))
        }
    }

    /// Receive data in "zero-copy" mode.  See `NNG_FLAG_ALLOC`.
    fn recv_zerocopy(&self) -> Result<mem::Alloc> {
        self.recv_zerocopy_flags(Default::default())
    }

    /// Receive data in "zero-copy" mode with [`Flags`](struct.Flags.html).
    fn recv_zerocopy_flags(&self, flags: Flags) -> Result<mem::Alloc> {
        let flags = (flags | Flags::ALLOC).bits();
        unsafe {
            let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
            let mut size: usize = 0;
            let ptr_ptr = (&mut ptr) as *mut _ as *mut core::ffi::c_void;
            let res = nng_recv(self.nng_socket(), ptr_ptr, &mut size, flags);
            let res = nng_int_to_result(res);
            res.map(|_| mem::Alloc::from_raw_parts(ptr, size))
        }
    }

    /// Receive a [`NngMsg`](../msg/struct.NngMsg.html): `recvmsg_flags(..., 0)`
    fn recvmsg(&self) -> Result<msg::NngMsg> {
        self.recvmsg_flags(Default::default())
    }

    /// Receive a [`NngMsg`](../msg/struct.NngMsg.html) with [`Flags`](struct.Flags.html).  See [nng_recvmsg](https://nanomsg.github.io/nng/man/v1.1.0/nng_recvmsg.3).
    fn recvmsg_flags(&self, flags: Flags) -> Result<msg::NngMsg> {
        unsafe {
            let mut recv_ptr: *mut nng_msg = std::ptr::null_mut();
            let res = nng_recvmsg(self.nng_socket(), &mut recv_ptr, flags.bits());
            Error::zero_map(res, || msg::NngMsg::from_raw(recv_ptr))
        }
    }
}

/// "Unsafe" version of `NngSocket`.  Merely wraps `nng_socket` and makes no attempt to manage the underlying resources.
/// May be invalid, close unexpectedly, etc.
#[derive(Debug)]
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

#[derive(Debug)]
struct InnerSocket {
    socket: nng_socket,
}

impl Drop for InnerSocket {
    fn drop(&mut self) {
        unsafe {
            trace!("Socket close: {:?}", self.socket);
            let res = nng_int_to_result(nng_close(self.socket));
            match res {
                Ok(()) => {}
                // Can't panic here.  Thrift's TIoChannel::split() clones the socket handle so we may get ECLOSED
                Err(Error::Errno(NngErrno::ECLOSED)) => {}
                Err(res) => {
                    debug!("nng_close {:?}", res);
                    panic!("nng_close {:?}", res);
                }
            }
        }
    }
}
