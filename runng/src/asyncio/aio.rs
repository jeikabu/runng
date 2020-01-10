//! Wrapper for `nng_aio`.

use super::*;
use std::{pin, ptr};

/// Type which exposes an [`NngAio`](struct.NngAio.html).
pub trait Aio {
    /// Obtain `NngAio`.
    fn aio(&self) -> &NngAio;
    fn aio_mut(&mut self) -> &mut NngAio;
}

/// Handle to `nng_aio`.  See [nng_aio](https://nng.nanomsg.org/man/v1.2.2/nng_aio.5).
///
/// # Safety
///
/// `nng_aio` only permits a single operation at a time.
/// Misusing this will cause undefined behavior.
#[derive(Debug)]
pub struct NngAio {
    aio: *mut nng_aio,
}

unsafe impl Send for NngAio {}

/// Type that is safe to pass as an argument to [`AioCallback`](type.AioCallback.html).
pub type AioArg<T> = pin::Pin<Box<T>>;
/// [`AioArg`](type.AioArg.html) as a raw pointer.
pub type AioArgPtr = *mut ::std::os::raw::c_void;
/// Type of function that is called when asynchronous I/O operation completes (or fails).
pub type AioCallback = unsafe extern "C" fn(arg1: AioArgPtr);

impl NngAio {
    /// Create an `nng_aio` for asynchronous I/O operations.
    /// See [nng_aio_alloc](https://nng.nanomsg.org/man/v1.2.2/nng_aio_alloc.3)
    pub fn create<T, F>(func: F, callback: AioCallback) -> Result<AioArg<T>>
    where
        T: Aio,
        F: FnOnce(NngAio) -> T,
    {
        let mut aio: *mut nng_aio = ptr::null_mut();
        let aio_wrapper = Self { aio };
        let mut aio_arg = Box::new(func(aio_wrapper));
        // This mess is needed to convert Box<_> to c_void
        let aio_arg_ptr = aio_arg.as_mut() as *mut _ as AioArgPtr;
        //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
        let res = unsafe { nng_aio_alloc(&mut aio, Some(callback), aio_arg_ptr) };
        nng_int_to_result(res)?;
        aio_arg.aio_mut().aio = aio;

        Ok(pin::Pin::from(aio_arg))
    }

    /// Obtain underlying `nng_aio`.
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn nng_aio(&self) -> *mut nng_aio {
        debug_assert!(!self.aio.is_null(), "nng_aio null");
        self.aio
    }

    /// Set scatter/gather vector for vectored I/O.
    ///
    /// See [nng_aio_set_iov](https://nng.nanomsg.org/man/v1.2.2/nng_aio_set_iov.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn set_iov(&self, iov: &[nng_iov]) -> Result<()> {
        let res = nng_aio_set_iov(self.nng_aio(), iov.len() as u32, iov.as_ptr());
        nng_int_to_result(res)
    }

    /// See [nng_aio_count](https://nng.nanomsg.org/man/v1.2.2/nng_aio_count.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn aio_count(&self) -> usize {
        nng_aio_count(self.nng_aio())
    }

    /// See [nng_aio_get_output](https://nng.nanomsg.org/man/v1.2.2/nng_aio_get_output.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn get_output(&self, index: u32) -> *mut ::std::os::raw::c_void {
        nng_aio_get_output(self.nng_aio(), index)
    }

    /// Cancel asynchronous I/O operation.
    /// See [nng_aio_cancel](https://nng.nanomsg.org/man/v1.2.2/nng_aio_cancel.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn cancel(&self) {
        nng_aio_cancel(self.nng_aio())
    }

    /// Cancel asynchronous I/O operation.
    /// See [nng_aio_abort](https://nng.nanomsg.org/man/v1.2.2/nng_aio_abort.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn abort_i32(&self, err: i32) {
        nng_aio_abort(self.nng_aio(), err)
    }

    /// Wait for an asynchronous I/O operation to complete.
    ///
    /// See [nng_aio_wait](https://nng.nanomsg.org/man/v1.2.2/nng_aio_wait.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn wait(&self) {
        nng_aio_wait(self.nng_aio())
    }

    /// Get result of asynchronous operation.
    ///
    /// See [nng_aio_result](https://nng.nanomsg.org/man/v1.2.2/nng_aio_result.3)
    ///
    /// # Safety
    ///
    /// See [`NngAio`](struct.NngAio.html)
    pub unsafe fn result(&self) -> Result<()> {
        let res = nng_aio_result(self.nng_aio());
        nng_int_to_result(res)
    }

    /// Set timeout for operations.
    /// See [nng_aio_set_timeout](https://nng.nanomsg.org/man/v1.2.2/nng_aio_set_timeout.3)
    pub fn set_timeout(&self, timeout: nng_duration) {
        unsafe {
            nng_aio_set_timeout(self.nng_aio(), timeout);
        }
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
