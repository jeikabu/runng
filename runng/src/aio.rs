use runng_sys::*;
use super::*;
use std::ptr;


pub trait Aio {
    fn aio(&self) -> &NngAio;
}

pub struct NngAio {
    aio: *mut nng_aio,
    // This isn't strictly correct from an NNG perspective.  It may be associated with:
    // - nng_context: nng_ctx_open(.., socket); nng_ctx_send(ctx, aio);
    // - nng_aio: nng_send_aio(socket, aio);
    socket: NngSocket,
}

pub type AioCallbackArg = *mut ::std::os::raw::c_void;
pub type AioCallback = unsafe extern "C" fn(arg1: AioCallbackArg);

impl NngAio {
    pub fn new(socket: NngSocket) -> NngAio {
        NngAio {
            aio: ptr::null_mut(),
            socket,
            }
    }
    pub fn init(&mut self, callback: AioCallback, arg: AioCallbackArg) -> NngReturn {
        unsafe {
            let mut aio: *mut nng_aio = ptr::null_mut();
            //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
            let res = nng_aio_alloc(&mut aio, Some(callback), arg);
            self.aio = aio;
            NngFail::from_i32(res)
        }
    }
    pub unsafe fn nng_aio(&self) -> *mut nng_aio {
        self.aio
    }
}

impl Drop for NngAio {
    fn drop(&mut self) {
        unsafe {
            debug!("NngAio.drop {:x}", self.aio as u64);
            if self.aio != ptr::null_mut() {
                nng_aio_free(self.aio);
            }
        }
    }
}

impl RawSocket for NngAio {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}

