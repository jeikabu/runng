use crate::*;
use runng_sys::*;
use std::{os::raw::c_void, ptr, slice};

#[derive(Debug)]
struct DroppableMsg {
    msg: *mut nng_msg,
}

unsafe impl Send for DroppableMsg {}
unsafe impl Sync for DroppableMsg {}

impl Drop for DroppableMsg {
    fn drop(&mut self) {
        unsafe {
            if !self.msg.is_null() {
                trace!("Dropping {:x}", self.msg as u64);
                nng_msg_free(self.msg);
            }
        }
    }
}

/// Wraps `nng_msg`.  See [nng_msg](https://nanomsg.github.io/nng/man/v1.1.0/nng_msg.5).
#[derive(Debug)]
pub struct NngMsg {
    msg: DroppableMsg,
}

impl NngMsg {
    /// Create a message.  See [nng_msg_alloc](https://nanomsg.github.io/nng/man/v1.1.0/nng_msg_alloc.3).
    pub fn create() -> NngResult<Self> {
        let mut msg: *mut nng_msg = ptr::null_mut();
        let res = unsafe { nng_msg_alloc(&mut msg, 0) };
        NngFail::succeed_then(res, || NngMsg::new_msg(msg))
    }

    pub fn new_msg(msg: *mut nng_msg) -> NngMsg {
        let msg = DroppableMsg { msg };
        NngMsg { msg }
    }

    pub fn take(mut self) -> *mut nng_msg {
        let msg = self.msg.msg;
        self.msg.msg = ptr::null_mut();
        msg
    }

    pub unsafe fn msg(&self) -> *mut nng_msg {
        self.msg.msg
    }

    pub fn header(&mut self) -> &[u8] {
        unsafe {
            let header = nng_msg_header(self.msg()) as *mut u8;
            let len = nng_msg_header_len(self.msg());
            slice::from_raw_parts(header, len)
        }
    }

    pub fn header_len(&self) -> usize {
        unsafe { nng_msg_header_len(self.msg()) }
    }

    pub fn body(&mut self) -> &[u8] {
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

    pub fn append(&mut self, data: *const u8, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_append(self.msg(), data as *const c_void, size)) }
    }

    pub fn insert(&mut self, data: *const u8, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_insert(self.msg(), data as *const c_void, size)) }
    }

    pub fn trim(&mut self, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_trim(self.msg(), size)) }
    }

    pub fn chop(&mut self, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_chop(self.msg(), size)) }
    }

    pub fn header_append(&mut self, data: *const u8, size: usize) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_msg_header_append(
                self.msg(),
                data as *const c_void,
                size,
            ))
        }
    }

    pub fn header_insert(&mut self, data: *const u8, size: usize) -> NngReturn {
        unsafe {
            NngFail::from_i32(nng_msg_header_insert(
                self.msg(),
                data as *const c_void,
                size,
            ))
        }
    }

    pub fn header_trim(&mut self, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_header_trim(self.msg(), size)) }
    }

    pub fn header_chop(&mut self, size: usize) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_header_chop(self.msg(), size)) }
    }

    pub fn append_u32(&mut self, data: u32) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_append_u32(self.msg(), data)) }
    }

    pub fn insert_u32(&mut self, data: u32) -> NngReturn {
        unsafe { NngFail::from_i32(nng_msg_insert_u32(self.msg(), data)) }
    }

    pub fn trim_u32(&mut self) -> NngResult<u32> {
        unsafe {
            let mut val: u32 = 0;
            NngFail::succeed(nng_msg_trim_u32(self.msg(), &mut val), val)
        }
    }

    pub fn chop_u32(&mut self) -> NngResult<u32> {
        unsafe {
            let mut val: u32 = 0;
            NngFail::succeed(nng_msg_chop_u32(self.msg(), &mut val), val)
        }
    }

    pub fn dup(&self) -> NngResult<NngMsg> {
        let mut msg: *mut nng_msg = ptr::null_mut();
        let res = unsafe { nng_msg_dup(&mut msg, self.msg()) };
        NngFail::succeed_then(res, || NngMsg::new_msg(msg))
    }

    pub fn clear(&mut self) {
        unsafe {
            nng_msg_clear(self.msg());
        }
    }
}
