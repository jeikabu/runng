//! Dialer

use runng_sys::*;
use std::sync::Arc;
use super::*;

/// Wraps `nng_dialer`.  See [nng_dialer](https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer.5).
pub struct NngDialer {
    dialer: nng_dialer,
    socket: Arc<NngSocket>
}

impl NngDialer {
    /// See [nng_dialer_create](https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer_create.3).
    pub(crate) fn new(socket: Arc<NngSocket>, url: &str) -> NngResult<NngDialer> {
        unsafe {
            let mut dialer = nng_dialer { id: 0 };
            let (_cstring, url) = to_cstr(url)?;
            NngFail::succeed(
                nng_dialer_create(&mut dialer, socket.nng_socket(), url), 
                NngDialer { dialer, socket }
            )
        }
    }

    // TODO: Use different type for started vs non-started dialer?  According to nng docs options can generally only
    // be set before the dialer is started.
    pub fn start(&self/*, flags: i32*/) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dialer_start(self.dialer, 0))
        }
    }
}

impl Opts for NngDialer {
    fn getopt_bool(&self, option: NngOption) -> NngResult<bool> {
        unsafe {
            let mut value: bool = Default::default();
            NngFail::succeed(nng_dialer_getopt_bool(self.dialer, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_int(&self, option: NngOption) -> NngResult<i32> {
        unsafe {
            let mut value: i32 = Default::default();
            NngFail::succeed(nng_dialer_getopt_int(self.dialer, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_size(&self, option: NngOption) -> NngResult<usize>
    {
        unsafe {
            let mut value: usize = Default::default();
            NngFail::succeed(nng_dialer_getopt_size(self.dialer, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_uint64(&self, option: NngOption) -> NngResult<u64> {
        unsafe {
            let mut value: u64 = Default::default();
            NngFail::succeed(nng_dialer_getopt_uint64(self.dialer, option.as_cptr(), &mut value), value)
        }
    }
    fn getopt_string(&self, option: NngOption) -> NngResult<NngString> {
        unsafe {
            let mut value: *mut ::std::os::raw::c_char = std::ptr::null_mut();
            let res = nng_dialer_getopt_string(self.dialer, option.as_cptr(), &mut value);
            NngFail::from_i32(res)?;
            Ok(NngString::new(value))
        }
    }

    fn setopt_bool(&mut self, option: NngOption, value: bool) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dialer_setopt_bool(self.dialer, option.as_cptr(), value))
        }
    }
    fn setopt_int(&mut self, option: NngOption, value: i32) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dialer_setopt_int(self.dialer, option.as_cptr(), value))
        }
    }
    fn setopt_size(&mut self, option: NngOption, value: usize) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dialer_setopt_size(self.dialer, option.as_cptr(), value))
        }
    }
    fn setopt_uint64(&mut self, option: NngOption, value: u64) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_dialer_setopt_uint64(self.dialer, option.as_cptr(), value))
        }
    }
    fn setopt_string(&mut self, option: NngOption, value: &str) -> NngReturn {
        unsafe {
            let (_, value) = to_cstr(value)?;
            NngFail::from_i32(nng_dialer_setopt_string(self.dialer, option.as_cptr(), value))
        }
    }
}