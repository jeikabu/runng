//! NNG strings.

use crate::{mem::Alloc, *};
use std::{ffi::CStr, ffi::CString, os::raw::c_char, result};

/// Handle to an owned NNG string.
/// See [nng_strfree](https://nanomsg.github.io/nng/man/v1.1.0/nng_strfree.3).
#[derive(Debug)]
pub struct NngString {
    pointer: *mut c_char,
}

impl NngString {
    /// Create a new string.
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        //let t = t.into();
        let bytes = CString::new(t)?.into_bytes_with_nul();
        let bytes = Alloc::new(bytes).ok_or(Error::Errno(NngErrno::ENOMEM))?;
        unsafe {
            let (ptr, _size) = bytes.take();
            Ok(NngString::from_raw(ptr as *mut _))
        }
    }
    /// Create a new string using `char*` obtained from NNG.
    ///
    /// # Safety
    ///
    /// Takes ownership of `pointer` and calls [nng_strfree](https://nanomsg.github.io/nng/man/v1.1.0/nng_strfree.3) when dropped.
    pub unsafe fn from_raw(pointer: *mut c_char) -> NngString {
        NngString { pointer }
    }

    /// Duplicate string.
    /// See [nng_strdup](https://nanomsg.github.io/nng/man/v1.1.0/nng_strdup.3)
    pub fn dup(&self) -> NngString {
        unsafe { NngString::from_raw(nng_strdup(self.pointer)) }
    }

    /// Helper that calls [Cstr::to_str()](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.to_str).
    pub fn to_str(&self) -> result::Result<&str, std::str::Utf8Error> {
        unsafe { CStr::from_ptr(self.pointer).to_str() }
    }
}

impl PartialEq for NngString {
    fn eq(&self, other: &NngString) -> bool {
        unsafe { CStr::from_ptr(self.pointer) == CStr::from_ptr(other.pointer) }
    }
}

impl Clone for NngString {
    fn clone(&self) -> Self {
        self.dup()
    }
}

impl Drop for NngString {
    fn drop(&mut self) {
        unsafe {
            nng_strfree(self.pointer);
        }
    }
}
