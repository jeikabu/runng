//! Asynchronous I/O

use super::*;
use runng_sys::*;
use std::ptr;
use std::sync::Arc;

/// Type which exposes a `NngAio`.
pub trait Aio {
    /// Obtain under-lying `NngAio`.
    fn aio(&self) -> &NngAio;
    fn aio_mut(&mut self) -> &mut NngAio;
}

/// Wraps `nng_aio`.  See [nng_aio](https://nanomsg.github.io/nng/man/v1.1.0/nng_aio.5).
pub struct NngAio {
    aio: *mut nng_aio,
    // This isn't strictly correct from an NNG perspective.  It may be associated with:
    // - nng_context: nng_ctx_open(.., socket); nng_ctx_send(ctx, aio);
    // - nng_aio: nng_send_aio(socket, aio);
    socket: Arc<NngSocket>,
}

unsafe impl Send for NngAio {}

pub type AioCallbackArg = *mut ::std::os::raw::c_void;
pub type AioCallback = unsafe extern "C" fn(arg1: AioCallbackArg);

impl NngAio {
    /// Create new `NngAio`.  Must call `init()`.
    pub fn new(socket: Arc<NngSocket>) -> NngAio {
        NngAio {
            aio: ptr::null_mut(),
            socket,
        }
    }

    /// Finish initialization of `nng_aio`.  See [nng_aio_alloc](https://nanomsg.github.io/nng/man/v1.1.0/nng_aio_alloc.3).
    pub fn init(&mut self, callback: AioCallback, arg: AioCallbackArg) -> NngReturn {
        unsafe {
            let mut aio: *mut nng_aio = ptr::null_mut();
            //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
            let res = nng_aio_alloc(&mut aio, Some(callback), arg);
            self.aio = aio;
            NngFail::from_i32(res)
        }
    }

    pub(crate) fn register_aio<T>(arg: T, callback: AioCallback) -> NngResult<Box<T>>
    where
        T: Aio,
    {
        let mut arg = Box::new(arg);
        // This mess is needed to convert Box<_> to c_void
        let arg_ptr = arg.as_mut() as *mut _ as AioCallbackArg;
        arg.aio_mut().init(callback, arg_ptr)?;
        Ok(arg)
    }

    /// Obtain underlying `nng_aio`.
    ///
    /// # Panics
    /// Will panic if `init()` was not called.
    pub unsafe fn nng_aio(&self) -> *mut nng_aio {
        if self.aio.is_null() {
            panic!("NngAio::init() not called");
        }
        self.aio
    }
}

impl Drop for NngAio {
    fn drop(&mut self) {
        unsafe {
            if !self.aio.is_null() {
                trace!("NngAio.drop {:x}", self.aio as u64);
                nng_aio_free(self.aio);
            }
        }
    }
}

impl InternalSocket for NngAio {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}
