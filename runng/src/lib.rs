extern crate runng_sys;

// public interface ISocketFactory
//     {
//         INngResult<IBusSocket> BusOpen();
//         INngResult<IReqSocket> RequesterOpen();
//         INngResult<IRepSocket> ReplierOpen();
//         INngResult<IPubSocket> PublisherOpen();
//         INngResult<ISubSocket> SubscriberOpen();
//         INngResult<IPushSocket> PusherOpen();
//         INngResult<IPullSocket> PullerOpen();
        
//         IListener ListenerCreate(ISocket socket, string url);
//         IDialer DialerCreate(ISocket socket, string url);

//         INngResult<TSocket> Dial<TSocket>(INngResult<TSocket> socketRes, string url) where TSocket : ISocket;
//         INngResult<TSocket> Listen<TSocket>(INngResult<TSocket> socketRes, string url) where TSocket : ISocket;
//     }

pub mod protocol;
pub mod transport;

use runng_sys::*;

use std::{
    error,
    fmt,
};

#[derive(Debug)]
pub enum NngError {
    EINTR        = nng_errno_enum_NNG_EINTR as isize,
    ENOMEM       = nng_errno_enum_NNG_ENOMEM as isize,
    EINVAL       = nng_errno_enum_NNG_EINVAL as isize,
    EBUSY        = nng_errno_enum_NNG_EBUSY as isize,
    ETIMEDOUT    = nng_errno_enum_NNG_ETIMEDOUT as isize,
    ECONNREFUSED = nng_errno_enum_NNG_ECONNREFUSED as isize,
    ECLOSED      = nng_errno_enum_NNG_ECLOSED as isize,
    EAGAIN       = nng_errno_enum_NNG_EAGAIN as isize,
    ENOTSUP      = nng_errno_enum_NNG_ENOTSUP as isize,
    EADDRINUSE   = nng_errno_enum_NNG_EADDRINUSE as isize,
    ESTATE       = nng_errno_enum_NNG_ESTATE as isize,
    ENOENT       = nng_errno_enum_NNG_ENOENT as isize,
    EPROTO       = nng_errno_enum_NNG_EPROTO as isize,
    EUNREACHABLE = nng_errno_enum_NNG_EUNREACHABLE as isize,
    EADDRINVAL   = nng_errno_enum_NNG_EADDRINVAL as isize,
    EPERM        = nng_errno_enum_NNG_EPERM as isize,
    EMSGSIZE     = nng_errno_enum_NNG_EMSGSIZE as isize,
    ECONNABORTED = nng_errno_enum_NNG_ECONNABORTED as isize,
    ECONNRESET   = nng_errno_enum_NNG_ECONNRESET as isize,
    ECANCELED    = nng_errno_enum_NNG_ECANCELED as isize,
    ENOFILES     = nng_errno_enum_NNG_ENOFILES as isize,
    ENOSPC       = nng_errno_enum_NNG_ENOSPC as isize,
    EEXIST       = nng_errno_enum_NNG_EEXIST as isize,
    EREADONLY    = nng_errno_enum_NNG_EREADONLY as isize,
    EWRITEONLY   = nng_errno_enum_NNG_EWRITEONLY as isize,
    ECRYPTO      = nng_errno_enum_NNG_ECRYPTO as isize,
    EPEERAUTH    = nng_errno_enum_NNG_EPEERAUTH as isize,
    ENOARG       = nng_errno_enum_NNG_ENOARG as isize,
    EAMBIGUOUS   = nng_errno_enum_NNG_EAMBIGUOUS as isize,
    EBADTYPE     = nng_errno_enum_NNG_EBADTYPE as isize,
    EINTERNAL    = nng_errno_enum_NNG_EINTERNAL as isize,
    ESYSERR      = nng_errno_enum_NNG_ESYSERR as isize,
    ETRANERR     = nng_errno_enum_NNG_ETRANERR as isize,
}

impl NngError {
    pub fn from_i32(value: i32) -> Option<NngError> {
        match value {
            value if value == NngError::EINTR as i32        => Some(NngError::EINTR),
            value if value == NngError::ENOMEM as i32       => Some(NngError::ENOMEM),
            value if value == NngError::EINVAL as i32       => Some(NngError::EINVAL),
            value if value == NngError::EBUSY as i32        => Some(NngError::EBUSY),
            value if value == NngError::ETIMEDOUT as i32    => Some(NngError::ETIMEDOUT),
            value if value == NngError::ECONNREFUSED as i32 => Some(NngError::ECONNREFUSED),
            value if value == NngError::ECLOSED as i32      => Some(NngError::ECLOSED),
            value if value == NngError::EAGAIN as i32       => Some(NngError::EAGAIN),
            value if value == NngError::ENOTSUP as i32      => Some(NngError::ENOTSUP),
            value if value == NngError::EADDRINUSE as i32   => Some(NngError::EADDRINUSE),
            value if value == NngError::ESTATE as i32       => Some(NngError::ESTATE),
            value if value == NngError::ENOENT as i32       => Some(NngError::ENOENT),
            value if value == NngError::EPROTO as i32       => Some(NngError::EPROTO),
            value if value == NngError::EUNREACHABLE as i32 => Some(NngError::EUNREACHABLE),
            value if value == NngError::EADDRINVAL as i32   => Some(NngError::EADDRINVAL),
            value if value == NngError::EPERM as i32        => Some(NngError::EPERM),
            value if value == NngError::EMSGSIZE as i32     => Some(NngError::EMSGSIZE),
            value if value == NngError::ECONNABORTED as i32 => Some(NngError::ECONNABORTED),
            value if value == NngError::ECONNRESET as i32   => Some(NngError::ECONNRESET),
            value if value == NngError::ECANCELED as i32    => Some(NngError::ECANCELED),
            value if value == NngError::ENOFILES as i32     => Some(NngError::ENOFILES),
            value if value == NngError::ENOSPC as i32       => Some(NngError::ENOSPC),
            value if value == NngError::EEXIST as i32       => Some(NngError::EEXIST),
            value if value == NngError::EREADONLY as i32    => Some(NngError::EREADONLY),
            value if value == NngError::EWRITEONLY as i32   => Some(NngError::EWRITEONLY),
            value if value == NngError::ECRYPTO as i32      => Some(NngError::ECRYPTO),
            value if value == NngError::EPEERAUTH as i32    => Some(NngError::EPEERAUTH),
            value if value == NngError::ENOARG as i32       => Some(NngError::ENOARG),
            value if value == NngError::EAMBIGUOUS as i32   => Some(NngError::EAMBIGUOUS),
            value if value == NngError::EBADTYPE as i32     => Some(NngError::EBADTYPE),
            value if value == NngError::EINTERNAL as i32    => Some(NngError::EINTERNAL),
            value if value == NngError::ESYSERR as i32      => Some(NngError::ESYSERR),
            value if value == NngError::ETRANERR as i32     => Some(NngError::ETRANERR),

            _        => None,
        }
    }
}

#[derive(Debug)]
pub enum NngFail {
    Err(NngError),
    Unknown(i32),
}

impl NngFail {
    pub fn from_i32(value: i32) -> NngFail {
        if let Some(error) = NngError::from_i32(value) {
            NngFail::Err(error)
        } else {
            NngFail::Unknown(value)
        }
    }
}

pub enum NngReturn {
    Ok,
    Fail(NngFail),
}

impl NngReturn {
    pub fn from_i32(value: i32) -> NngReturn {
        if value == 0 {
            NngReturn::Ok
        } else {
            NngReturn::Fail(NngFail::from_i32(value))
        }
    }

    pub fn from<T>(return_value: i32, result: T) -> NngResult<T> {
        if return_value == 0 {
            Ok(result)
        } else {
            Err(NngFail::from_i32(return_value))
        }
    }
}

impl error::Error for NngError {
}
impl fmt::Display for NngError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self,)
    }
}

type NngResult<T> = Result<T, NngFail>;

pub trait Factory {
    fn requester_open(&self) -> NngResult<protocol::Req0>;
    fn replier_open(&self) -> NngResult<protocol::Rep0>;
}

pub trait Socket {
    fn socket(&self) -> nng_socket;
}

pub struct NngSocket {
    socket: nng_socket,
}

impl NngSocket {
    fn new() -> NngSocket {
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
                panic!(res);
            }
        }
    }
}

pub struct Latest {
}

impl Latest {
    pub fn new() -> Latest {
        Latest {}
    }
}

impl Factory for Latest {
    fn requester_open(&self) -> NngResult<protocol::Req0> {
        protocol::Req0::open()
    }
    fn replier_open(&self) -> NngResult<protocol::Rep0> {
        protocol::Rep0::open()
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

use std::ffi::CString;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
    }
}
