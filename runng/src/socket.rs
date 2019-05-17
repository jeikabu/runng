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

impl GetOpts for NngSocket {
    fn get_bool(&self, option: NngOption) -> Result<bool> {
        unsafe {
            let mut value: bool = Default::default();
            Error::zero_map(
                nng_socket_get_bool(self.nng_socket(), option.as_cptr(), &mut value),
                || value,
            )
        }
    }
    fn get_int(&self, option: NngOption) -> Result<i32> {
        unsafe {
            let mut value: i32 = Default::default();
            Error::zero_map(
                nng_socket_get_int(self.nng_socket(), option.as_cptr(), &mut value),
                || value,
            )
        }
    }
    fn get_ms(&self, option: NngOption) -> Result<i32> {
        unsafe {
            let mut value: i32 = Default::default();
            Error::zero_map(
                nng_socket_get_ms(self.nng_socket(), option.as_cptr(), &mut value),
                || value,
            )
        }
    }
    fn get_size(&self, option: NngOption) -> Result<usize> {
        unsafe {
            let mut value: usize = Default::default();
            Error::zero_map(
                nng_socket_get_size(self.nng_socket(), option.as_cptr(), &mut value),
                || value,
            )
        }
    }
    fn get_uint64(&self, option: NngOption) -> Result<u64> {
        unsafe {
            let mut value: u64 = Default::default();
            Error::zero_map(
                nng_socket_get_uint64(self.nng_socket(), option.as_cptr(), &mut value),
                || value,
            )
        }
    }
    fn get_string(&self, option: NngOption) -> Result<NngString> {
        unsafe {
            let mut value: *mut ::std::os::raw::c_char = std::ptr::null_mut();
            let res = nng_socket_get_string(self.nng_socket(), option.as_cptr(), &mut value);
            nng_int_to_result(res)?;
            Ok(NngString::from_raw(value))
        }
    }
}

impl SetOpts for NngSocket {
    fn set_bool(&mut self, option: NngOption, value: bool) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_socket_set_bool(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
    fn set_int(&mut self, option: NngOption, value: i32) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_socket_set_int(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
    fn set_ms(&mut self, option: NngOption, value: nng_duration) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_socket_set_ms(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
    fn set_size(&mut self, option: NngOption, value: usize) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_socket_set_size(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
    fn set_uint64(&mut self, option: NngOption, value: u64) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_socket_set_uint64(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
    fn set_string(&mut self, option: NngOption, value: &str) -> Result<()> {
        unsafe {
            let (_, value) = to_cstr(value)?;
            nng_int_to_result(nng_socket_set_string(
                self.nng_socket(),
                option.as_cptr(),
                value,
            ))
        }
    }
}

impl Socket for NngSocket {
    fn socket(&self) -> &NngSocket {
        self
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        self
    }
}

impl SendSocket for NngSocket {}
impl RecvSocket for NngSocket {}

impl Clone for NngSocket {
    fn clone(&self) -> Self {
        let socket = self.socket.clone();
        Self { socket }
    }
}

/// Type which exposes a [`NngSocket`](struct.NngSocket.html).
pub trait Socket: Sized {
    /// Obtain underlying `NngSocket`.
    fn socket(&self) -> &NngSocket;
    fn socket_mut(&mut self) -> &mut NngSocket;
    /// Obtain underlying `nng_socket`.
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

/// `Socket` that can accept connections from ("listen" to) other `Socket`s.
pub trait Listen: Socket {
    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen(self, url: &str) -> Result<Self> {
        self.listen_flags(url, Default::default())
    }

    /// Listen for connections to specified URL.  See [nng_listen](https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3).
    fn listen_flags(self, url: &str, flags: SocketFlags) -> Result<Self> {
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
    fn dial(self, url: &str) -> Result<Self> {
        self.dial_flags(url, Default::default())
    }

    /// Dial socket specified by URL.  See [nng_dial](https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3)
    fn dial_flags(self, url: &str, flags: SocketFlags) -> Result<Self> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_dial(self.nng_socket(), ptr, std::ptr::null_mut(), flags.bits());
            Error::zero_map(res, || self)
        }
    }

    fn dial_mut(&mut self, url: &str) -> Result<()> {
        unsafe {
            let (_cstring, ptr) = to_cstr(url)?;
            let res = nng_dial(
                self.nng_socket(),
                ptr,
                std::ptr::null_mut(),
                SocketFlags::default().bits(),
            );
            nng_int_to_result(res)
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
