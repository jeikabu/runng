//! Pipe

#![cfg(feature = "pipes")]

use super::*;
use crate::{dialer::UnsafeDialer, listener::UnsafeListener, msg::NngMsg};
use runng_derive::NngGetOpts;
use runng_sys::*;

pub type PipeNotifyCallback =
    unsafe extern "C" fn(pipe: nng_pipe, event: i32, arg1: PipeNotifyCallbackArg);
pub type PipeNotifyCallbackArg = *mut ::std::os::raw::c_void;

/// Wraps `nng_pipe`.  See [nng_pipe](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe.5).
#[derive(NngGetOpts)] // Note: nng_pipe has no setopt() functions
#[prefix = "nng_pipe_"]
pub struct NngPipe {
    #[nng_member]
    pipe: nng_pipe,
}

impl NngPipe {
    /// Get pipe associated with a message, if one exists.  See [nng_msg_get_pipe](https://nanomsg.github.io/nng/man/v1.1.0/nng_msg_get_pipe.3).
    pub(crate) fn create(message: &NngMsg) -> Option<Self> {
        unsafe {
            let pipe = nng_msg_get_pipe(message.msg());
            if (pipe.id as i32) < 0 {
                None
            } else {
                Some(NngPipe { pipe })
            }
        }
    }

    /// See [nng_pipe_id](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_pipe_id(self.pipe) }
    }

    /// Obtain underlying `nng_pipe`
    pub unsafe fn nng_pipe(&self) -> nng_pipe {
        self.pipe
    }

    /// Get socket that owns the pipe.  See [nng_pipe_socket](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_socket.3).
    pub unsafe fn socket(&self) -> Option<UnsafeSocket> {
        let socket = UnsafeSocket::new(nng_pipe_socket(self.pipe));
        if socket.id() == -1 {
            None
        } else {
            Some(socket)
        }
    }

    /// Get dialer that created the pipe.  See [nng_pipe_dialer](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_dialer.3).
    pub unsafe fn dialer(&self) -> Option<UnsafeDialer> {
        let dialer = UnsafeDialer::new(nng_pipe_dialer(self.pipe));
        if dialer.id() == -1 {
            None
        } else {
            Some(dialer)
        }
    }

    /// Get listener that created the pipe.  See [nng_pipe_listener](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_listener.3).
    pub unsafe fn listener(&self) -> Option<UnsafeListener> {
        let listener = UnsafeListener::new(nng_pipe_listener(self.pipe));
        if listener.id() == -1 {
            None
        } else {
            Some(listener)
        }
    }

    /// Closes the pipe.  See [nng_pipe_close](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_close.3).
    /// This will cause associated aio/ctx functions that were using the pipe to fail.
    pub unsafe fn close(self) -> NngReturn {
        NngFail::from_i32(nng_pipe_close(self.pipe))
    }
}
