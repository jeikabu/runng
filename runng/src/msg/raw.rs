use runng_sys::*;
use std::{
    os::raw::c_void,
    ptr,
    slice,
};
use super::*;

pub struct NngMsg {
    msg: *mut nng_msg
}

impl NngMsg {
    pub fn new() -> NngResult<NngMsg> {
        let mut msg: *mut nng_msg = ptr::null_mut();
        let res = unsafe {
            nng_msg_alloc(&mut msg, 0)
        };
        NngReturn::from(res, NngMsg {
            msg
        })
    }

    pub fn header(&mut self) -> &[u8] {
        unsafe {
            let header = nng_msg_header(self.msg) as *mut u8;
            let len = nng_msg_header_len(self.msg);
            slice::from_raw_parts(header, len)
        }
    }
    
    pub fn header_len(&self) -> usize {
        unsafe {
            nng_msg_header_len(self.msg)
        }
    }

    pub fn body(&mut self) -> &[u8] {
        unsafe {
            let body = nng_msg_body(self.msg) as *mut u8;
            let len = nng_msg_len(self.msg);
            slice::from_raw_parts(body, len)
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            nng_msg_len(self.msg)
        }
    }

    pub fn append(&mut self, data: *const c_void, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_append(self.msg, data, size), ())
        }
    }

    pub fn insert(&mut self, data: *const c_void, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_insert(self.msg, data, size), ())
        }
    }

    pub fn trim(&mut self, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_trim(self.msg, size), ())
        }
    }

    pub fn chop(&mut self, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_chop(self.msg, size), ())
        }
    }

    pub fn header_append(&mut self, data: *const c_void, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_header_append(self.msg, data, size), ())
        }
    }

    pub fn header_insert(&mut self, data: *const c_void, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_header_insert(self.msg, data, size), ())
        }
    }

    pub fn header_trim(&mut self, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_header_trim(self.msg, size), ())
        }
    }

    pub fn header_chop(&mut self, size: usize) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_header_chop(self.msg, size), ())
        }
    }

    pub fn append_u32(&mut self, data: u32) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_append_u32(self.msg, data), ())
        }
    }

    pub fn insert_u32(&mut self, data: u32) -> NngResult<()> {
        unsafe {
            NngReturn::from(nng_msg_insert_u32(self.msg, data), ())
        }
    }

    pub fn trim_u32(&mut self) -> NngResult<u32> {
        unsafe {
            let mut val: u32 = 0;
            NngReturn::from(nng_msg_trim_u32(self.msg, &mut val), val)
        }
    }

    pub fn chop_u32(&mut self) -> NngResult<u32> {
        unsafe {
            let mut val: u32 = 0;
            NngReturn::from(nng_msg_chop_u32(self.msg, &mut val), val)
        }
    }

    pub fn dup(&self) -> NngResult<NngMsg> {
        let mut msg: *mut nng_msg = ptr::null_mut();
        let res = unsafe {
            nng_msg_dup(&mut msg, self.msg)
        };
        NngReturn::from(res, NngMsg {
            msg
        })
    }

    pub fn clear(&mut self) {
        unsafe {
            nng_msg_clear(self.msg);
        }
    }
}

impl Drop for NngMsg {
    fn drop(&mut self) {
        unsafe {
            nng_msg_free(self.msg);
        }
    }
}