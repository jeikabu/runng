//! Messages.

use crate::*;
use runng_derive::NngMsgOpts;
use runng_sys::*;
use std::{os::raw::c_void, ptr, slice};

/// Wraps `nng_msg`.  See [nng_msg](https://nng.nanomsg.org/man/v1.2.2/nng_msg.5).
#[derive(Debug, NngMsgOpts)]
pub struct NngMsg {
    msg: *mut nng_msg,
}

impl NngMsg {
    /// Create a message.  See [nng_msg_alloc](https://nng.nanomsg.org/man/v1.2.2/nng_msg_alloc.3).
    pub fn new() -> Result<Self> {
        NngMsg::with_capacity(0)
    }

    /// Create a message with body length `size_bytes`.  See [nng_msg_alloc](https://nng.nanomsg.org/man/v1.2.2/nng_msg_alloc.3).
    pub fn with_capacity(size_bytes: usize) -> Result<Self> {
        unsafe {
            let mut msg: *mut nng_msg = ptr::null_mut();
            let res = nng_msg_alloc(&mut msg, size_bytes);
            nng_int_to_result(res).map(|_| NngMsg::from_raw(msg))
        }
    }

    /// Create a message using pointer from NNG.
    ///
    /// # Safety
    ///
    /// Takes ownership of `msg` and releases it when dropped.
    pub unsafe fn from_raw(msg: *mut nng_msg) -> NngMsg {
        NngMsg { msg }
    }

    /// Take ownership of the contained nng_msg.  You are responsible for calling `nng_msg_free`.
    pub unsafe fn take(mut self) -> *mut nng_msg {
        let msg = self.msg;
        self.msg = ptr::null_mut();
        msg
    }

    pub unsafe fn msg(&self) -> *mut nng_msg {
        self.msg
    }

    pub fn as_slice(&self) -> &[u8] {
        self.body()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let body = nng_msg_body(self.msg()) as *mut u8;
            let len = nng_msg_len(self.msg());
            slice::from_raw_parts_mut(body, len)
        }
    }

    pub fn header(&self) -> &[u8] {
        unsafe {
            let header = nng_msg_header(self.msg()) as *mut u8;
            let len = nng_msg_header_len(self.msg());
            slice::from_raw_parts(header, len)
        }
    }

    pub fn header_len(&self) -> usize {
        unsafe { nng_msg_header_len(self.msg()) }
    }

    pub fn body(&self) -> &[u8] {
        unsafe {
            let body = nng_msg_body(self.msg()) as *mut u8;
            let len = nng_msg_len(self.msg());
            slice::from_raw_parts(body, len)
        }
    }

    pub fn len(&self) -> usize {
        unsafe { nng_msg_len(self.msg()) }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { nng_msg_len(self.msg()) == 0 }
    }

    pub fn append_slice(&mut self, data: &[u8]) -> Result<()> {
        self.append_ptr(data.as_ptr(), data.len())
    }

    pub fn append_ptr(&mut self, data: *const u8, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_append(self.msg(), data as *const c_void, size)) }
    }

    pub fn insert_slice(&mut self, data: &[u8]) -> Result<()> {
        self.insert_ptr(data.as_ptr(), data.len())
    }

    pub fn insert_ptr(&mut self, data: *const u8, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_insert(self.msg(), data as *const c_void, size)) }
    }

    pub fn trim(&mut self, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_trim(self.msg(), size)) }
    }

    pub fn chop(&mut self, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_chop(self.msg(), size)) }
    }

    pub fn header_append_slice(&mut self, data: &[u8]) -> Result<()> {
        self.header_append_ptr(data.as_ptr(), data.len())
    }

    pub fn header_append_ptr(&mut self, data: *const u8, size: usize) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_msg_header_append(
                self.msg(),
                data as *const c_void,
                size,
            ))
        }
    }

    pub fn header_insert_slice(&mut self, data: &[u8]) -> Result<()> {
        self.header_insert_ptr(data.as_ptr(), data.len())
    }

    pub fn header_insert_ptr(&mut self, data: *const u8, size: usize) -> Result<()> {
        unsafe {
            nng_int_to_result(nng_msg_header_insert(
                self.msg(),
                data as *const c_void,
                size,
            ))
        }
    }

    pub fn header_trim(&mut self, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_header_trim(self.msg(), size)) }
    }

    pub fn header_chop(&mut self, size: usize) -> Result<()> {
        unsafe { nng_int_to_result(nng_msg_header_chop(self.msg(), size)) }
    }

    pub fn dup(&self) -> Result<NngMsg> {
        unsafe {
            let mut msg: *mut nng_msg = ptr::null_mut();
            let res = nng_msg_dup(&mut msg, self.msg());
            nng_int_to_result(res).map(|_| NngMsg::from_raw(msg))
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            nng_msg_clear(self.msg());
        }
    }

    pub fn get_pipe(&self) -> Option<pipe::NngPipe> {
        pipe::NngPipe::new(self)
    }

    pub fn set_pipe(&mut self, pipe: &pipe::NngPipe) {
        unsafe {
            nng_msg_set_pipe(self.msg(), pipe.nng_pipe());
        }
    }
}

impl AsRef<[u8]> for NngMsg {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for NngMsg {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl Clone for NngMsg {
    fn clone(&self) -> Self {
        self.dup().unwrap()
    }
}

// TODO: ideally we'd replace `*mut XXX` with Unique<>, but seems that will never stabilize:
// https://github.com/rust-lang/rust/issues/27730
// Implement Send/Sync for now...
unsafe impl Send for NngMsg {}
unsafe impl Sync for NngMsg {}

impl PartialEq for NngMsg {
    fn eq(&self, other: &NngMsg) -> bool {
        self.header() == other.header() && self.body() == other.body()
    }
}

impl Drop for NngMsg {
    fn drop(&mut self) {
        unsafe {
            if !self.msg.is_null() {
                trace!("Dropping {:x}", self.msg as u64);
                nng_msg_free(self.msg);
            }
        }
    }
}
