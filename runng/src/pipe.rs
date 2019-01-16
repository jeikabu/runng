//! Pipe

#![cfg(feature = "pipes")]

use super::*;
use crate::{dialer::UnsafeDialer, listener::UnsafeListener, msg::NngMsg};
use runng_derive::NngGetOpts;
use runng_sys::*;

/// Pipe events.  See [nng_pipe_notify](https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe_notify.3).
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum PipeEvent {
    /// This event occurs after a connection and negotiation has completed, but before the pipe is added to the socket.
    AddPre = nng_pipe_ev_NNG_PIPE_EV_ADD_PRE,
    /// This event occurs after the pipe is fully added to the socket.
    /// Prior to this time, it is not possible to communicate over the pipe with the socket.
    AddPost = nng_pipe_ev_NNG_PIPE_EV_ADD_POST,
    /// This event occurs after the pipe has been removed from the socket.
    /// The underlying transport may be closed at this point, and it is not possible communicate using this pipe.
    RemPost = nng_pipe_ev_NNG_PIPE_EV_REM_POST,
}

impl PipeEvent {
    pub fn from_i32(value: i32) -> Option<PipeEvent> {
        match value {
            value if value == PipeEvent::AddPre as i32 => Some(PipeEvent::AddPre),
            value if value == PipeEvent::AddPost as i32 => Some(PipeEvent::AddPost),
            value if value == PipeEvent::RemPost as i32 => Some(PipeEvent::RemPost),
            _ => None,
        }
    }
}

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
