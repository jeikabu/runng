//! Pipe

#![cfg(feature = "pipes")]

use super::*;
use crate::{dialer::UnsafeDialer, listener::UnsafeListener, msg::NngMsg};
use runng_derive::NngGetOpts;
use runng_sys::*;

pub type PipeNotifyCallback =
    unsafe extern "C" fn(pipe: nng_pipe, event: nng_pipe_ev, arg1: PipeNotifyCallbackArg);
pub type PipeNotifyCallbackArg = *mut ::std::os::raw::c_void;

/// Pipe events that can occur on sockets.
/// See [`nng_pipe_ev` in nng_pipe_notify](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_notify.3.html).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum NngPipeEv {
    AddPre = NNG_PIPE_EV_ADD_PRE,
    AddPost = NNG_PIPE_EV_ADD_POST,
    RemPost = NNG_PIPE_EV_REM_POST,
}

impl core::convert::TryFrom<nng_pipe_ev> for NngPipeEv {
    type Error = EnumFromIntError;

    fn try_from(value: nng_pipe_ev) -> std::result::Result<Self, Self::Error> {
        use NngPipeEv::*;
        match value {
            NNG_PIPE_EV_ADD_PRE => Ok(AddPre),
            NNG_PIPE_EV_ADD_POST => Ok(AddPost),
            NNG_PIPE_EV_REM_POST => Ok(RemPost),
            _ => Err(EnumFromIntError(value as i32)),
        }
    }
}

/// Wraps `nng_pipe`.  See [nng_pipe](https://nng.nanomsg.org/man/v1.2.2/nng_pipe.5).
#[derive(Debug, NngGetOpts)] // Note: nng_pipe has no setopt() functions
#[prefix = "nng_pipe_"]
pub struct NngPipe {
    pipe: nng_pipe,
}

impl NngPipe {
    /// Get pipe associated with a message, if one exists.  See [nng_msg_get_pipe](https://nng.nanomsg.org/man/v1.2.2/nng_msg_get_pipe.3).
    pub(crate) fn new(message: &NngMsg) -> Option<Self> {
        unsafe {
            let pipe = nng_msg_get_pipe(message.msg());
            let id = nng_pipe_id(pipe);
            if id <= 0 {
                None
            } else {
                Some(NngPipe { pipe })
            }
        }
    }

    /// See [nng_pipe_id](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_pipe_id(self.pipe) }
    }

    /// Obtain underlying `nng_pipe`
    pub unsafe fn nng_pipe(&self) -> nng_pipe {
        self.pipe
    }

    /// Get socket that owns the pipe.  See [nng_pipe_socket](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_socket.3).
    pub unsafe fn socket(&self) -> Option<UnsafeSocket> {
        let socket = UnsafeSocket::new(nng_pipe_socket(self.pipe));
        if socket.id() == -1 {
            None
        } else {
            Some(socket)
        }
    }

    /// Get dialer that created the pipe.  See [nng_pipe_dialer](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_dialer.3).
    pub unsafe fn dialer(&self) -> Option<UnsafeDialer> {
        let dialer = UnsafeDialer::new(nng_pipe_dialer(self.pipe));
        if dialer.id() == -1 {
            None
        } else {
            Some(dialer)
        }
    }

    /// Get listener that created the pipe.  See [nng_pipe_listener](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_listener.3).
    pub unsafe fn listener(&self) -> Option<UnsafeListener> {
        let listener = UnsafeListener::new(nng_pipe_listener(self.pipe));
        if listener.id() == -1 {
            None
        } else {
            Some(listener)
        }
    }

    /// Closes the pipe.  See [nng_pipe_close](https://nng.nanomsg.org/man/v1.2.2/nng_pipe_close.3).
    /// This will cause associated aio/ctx functions that were using the pipe to fail.
    pub unsafe fn close(self) -> Result<()> {
        nng_int_to_result(nng_pipe_close(self.pipe))
    }

    #[cfg(feature = "unstable")]
    pub fn getopt_sockaddr(&self, option: NngOption) -> Result<nng_sockaddr> {
        unsafe {
            let mut sockaddr = nng_sockaddr::default();
            Error::zero_map(
                nng_pipe_getopt_sockaddr(self.pipe, option.as_cptr(), &mut sockaddr),
                || sockaddr,
            )
        }
    }
}

impl NngWrapper for NngPipe {
    type NngType = nng_pipe;
    unsafe fn get_nng_type(&self) -> Self::NngType {
        self.pipe
    }
}
