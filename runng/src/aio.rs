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

type AioCallbackArg = *mut ::std::os::raw::c_void;
type AioCallback = unsafe extern "C" fn(arg1: AioCallbackArg);

impl NngAio {
    pub fn new(socket: NngSocket, callback: AioCallback, arg: AioCallbackArg) -> NngResult<NngAio> {
        unsafe {
            let mut tmp_aio: *mut nng_aio = ptr::null_mut();
            //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
            let res = nng_aio_alloc(&mut tmp_aio, Some(callback), arg);
            if res != 0 {
                Err(NngFail::from_i32(res))
            } else {
                let aio = NngAio {
                    aio: tmp_aio,
                    socket
                };
                NngReturn::from(res, aio)
            }
        }
    }
}

impl Drop for NngAio {
    fn drop(&mut self) {
        unsafe {
            nng_aio_free(self.aio);
        }
    }
}

impl RawSocket for NngAio {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Aio for NngAio {
    fn aio(&self) -> *mut nng_aio {
        self.aio
    }
}