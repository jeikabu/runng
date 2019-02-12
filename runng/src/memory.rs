use core::ffi::c_void;
use runng_sys::*;

/// Wraps memory allocated with `nng_alloc`.  See [nng_alloc](https://nanomsg.github.io/nng/man/v1.1.0/nng_alloc.3) and [nng_free](https://nanomsg.github.io/nng/man/v1.1.0/nng_free.3).
#[derive(Debug)]
pub struct Alloc {
    ptr: *mut c_void,
    size: usize,
}

impl Alloc {
    pub fn create(size: usize) -> Option<Alloc> {
        unsafe {
            let ptr = nng_alloc(size);
            if ptr.is_null() {
                None
            } else {
                Some(Alloc::create_raw(ptr, size))
            }
        }
    }

    pub(crate) unsafe fn create_raw(ptr: *mut c_void, size: usize) -> Alloc {
        Alloc { ptr, size }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr as *mut u8, self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut u8, self.size) }
    }

    /// Take ownership of the contained memory.  You are responsible for calling `nng_free`, or passing it to nng_send, etc.
    pub unsafe fn take(mut self) -> (*mut c_void, usize) {
        self.take_unsafe()
    }

    unsafe fn take_unsafe(&mut self) -> (*mut c_void, usize) {
        let retval = (self.ptr, self.size);
        self.ptr = std::ptr::null_mut();
        self.size = 0;
        retval
    }
}

impl AsRef<[u8]> for Alloc {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for Alloc {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl Clone for Alloc {
    fn clone(&self) -> Self {
        let src = self.as_slice();
        let mut clone = Alloc::create(src.len()).unwrap();
        let dest = clone.as_mut_slice();
        dest.copy_from_slice(src);
        clone
    }
}

impl Drop for Alloc {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let (ptr, size) = self.take_unsafe();
                nng_free(ptr, size);
            }
        }
    }
}

impl PartialEq for Alloc {
    fn eq(&self, other: &Alloc) -> bool {
        self.as_slice() == other.as_slice()
    }
}
