use runng_sys::*;
use super::*;
use std::ptr;

pub struct NngAio {
    aio: *mut nng_aio,
    socket: NngSocket,
}

pub trait Aio {
    fn aio(&self) -> *mut nng_aio;
}

pub type AioCallbackArg = *mut ::std::os::raw::c_void;
pub type AioCallback = unsafe extern "C" fn(arg1: AioCallbackArg);

impl NngAio {
    pub fn new(socket: NngSocket, callback: AioCallback, arg: AioCallbackArg) -> NngResult<NngAio> {
        unsafe {
            let mut aio: *mut nng_aio = ptr::null_mut();
            //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
            let res = nng_aio_alloc(&mut aio, Some(callback), arg);
            NngFail::succeed_then(res, || NngAio { aio, socket })
        }
    }
}

impl Drop for NngAio {
    fn drop(&mut self) {
        unsafe {
            println!("Drop aio {:x}", self.aio as u64);
            nng_aio_free(self.aio);
        }
    }
}

impl RawSocket for NngAio {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}

impl Aio for NngAio {
    fn aio(&self) -> *mut nng_aio {
        self.aio
    }
}