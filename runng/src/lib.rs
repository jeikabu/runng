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

mod protocols;
mod transports;

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

// #[derive(Debug)]
// pub struct NngError (NngErrorCode);

impl error::Error for NngError {

}
impl fmt::Display for NngError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self,)
    }
}

type NngResult<T> = Result<T, NngError>;

pub trait Factory {
    fn requester_open(&self) -> NngResult<u32>;
}

pub struct Socket {
    socket: nng_socket,
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            nng_close(self.socket);
        }
    }
}

pub struct Latest {
}

impl Latest {
    fn new() -> Latest {
        Latest {}
    }
}

impl Factory for Latest {
    fn requester_open(&self) -> NngResult<u32> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let factory = Latest::new();
        factory.requester_open().unwrap();
    }
}
