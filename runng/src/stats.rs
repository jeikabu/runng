/*!
Runtime statistics

Statistics are organized as a tree.  Starting at the root, nodes may have a child whose siblings are likewise children of the parent.

The lifetime of the children is bound to that of the root.  This won't compile:
```compile_fail
use runng::stats::*;
let mut child: Option<NngStatChild> = None;
{
    let root = NngStatRoot::create().unwrap();
    child = root.child();
}
println!("Name = {}", child.unwrap().name().unwrap());
```

## Examples

```rust
use log::{debug};
use runng::{
    *,
    stats::NngStat,
    stats::NngStatRoot,
};

#[test]
fn stats_example() -> Result<()> {
    // https://github.com/nanomsg/nng/issues/841
    let url = "inproc://test";
    let factory = ProtocolFactory::default();
    let _pusher = factory.pusher_open()?.listen(&url)?;
    let _puller = factory.puller_open()?.dial(&url)?;

    let stats = NngStatRoot::new()?;
    let child = stats.child().unwrap();
    for stat in child.iter() {
        debug!("{}", stat.name().unwrap());
    }
    Ok(())
}
```
*/

use crate::*;
use log::trace;
use runng_sys::*;
use std::{ffi::CStr, marker, result};

pub trait NngStat {
    /// Obtain underlying [`nng_stat`](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat.5).
    unsafe fn nng_stat(&self) -> *mut nng_stat;
    /// Returns the first child statistic, if any.  See [nng_stat_child](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_child.3).
    fn child(&self) -> Option<NngStatChild> {
        unsafe {
            let node = nng_stat_child(self.nng_stat());
            NngStatChild::new(node)
        }
    }
}

/* Root of tree of statistics snapshot.
## Examples
```rust
# use runng::stats::*;
let child = NngStatRoot::new().unwrap().child();
```
*/
#[derive(Debug)]
pub struct NngStatRoot {
    node: *mut nng_stat,
}

impl NngStatRoot {
    /// Get statistics snapshot.  See [nng_stats_get](https://nanomsg.github.io/nng/man/v1.1.0/nng_stats_get.3).
    pub fn create() -> Result<NngStatRoot> {
        unsafe {
            let mut node: *mut nng_stat = std::ptr::null_mut();
            let res = nng_stats_get(&mut node);
            Error::zero_map(res, || NngStatRoot { node })
        }
    }
}

impl NngStat for NngStatRoot {
    unsafe fn nng_stat(&self) -> *mut nng_stat {
        self.node
    }
}

impl Drop for NngStatRoot {
    fn drop(&mut self) {
        unsafe {
            trace!("Drop NngStatRoot");
            nng_stats_free(self.node)
        }
    }
}

#[derive(Debug)]
pub struct NngStatChild<'root> {
    node: *mut nng_stat,
    _phantom: marker::PhantomData<&'root nng_stat>,
}

/// Child of statistic node in tree of statistics.  See `NngStat::child()`.
impl<'root> NngStatChild<'root> {
    pub fn new(node: *mut nng_stat) -> Option<NngStatChild<'root>> {
        if node.is_null() {
            None
        } else {
            Some(NngStatChild {
                node,
                _phantom: marker::PhantomData,
            })
        }
    }

    /// See [nng_stat_name](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_name.3).
    pub fn name(&self) -> result::Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_name(self.nng_stat());
            CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_desc](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_desc.3).
    pub fn desc(&self) -> result::Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_desc(self.nng_stat());
            CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3).
    pub fn stat_type(&self) -> result::Result<nng_stat_type_enum, TryFromIntError> {
        unsafe {
            let val = nng_stat_type(self.nng_stat());
            nng_stat_type_enum::try_from(val)
        }
    }

    /// See [nng_stat_value](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn value(&self) -> u64 {
        unsafe { nng_stat_value(self.nng_stat()) }
    }

    /// If the statistic type is of type `NNG_STAT_STRING` returns the string value.
    /// Otherwise, `None` is returned.
    /// See [nng_stat_string](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_string.3).
    pub fn string(&self) -> Option<&str> {
        unsafe {
            let ptr = nng_stat_string(self.nng_stat());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    /// See [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3).
    pub fn unit(&self) -> result::Result<nng_unit_enum, TryFromIntError> {
        unsafe {
            let val = nng_stat_unit(self.nng_stat());
            nng_unit_enum::try_from(val)
        }
    }

    /// See [nng_stat_timestamp](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn timestamp(&self) -> u64 {
        unsafe { nng_stat_timestamp(self.nng_stat()) }
    }

    /// Returns an iterator over self and sibling statistics.  See [nng_stat_next](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_next.3).
    pub fn iter(&self) -> Iter {
        unsafe {
            let node = self.nng_stat();
            Iter {
                node: NngStatChild::new(node),
            }
        }
    }

    // The explicit `'root` lifetime is important here so the lifetime is the
    // top-level `NngStatRoot` rather than &self.
    /// Get the next sibling statistic.  See [nng_stat_next](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_next.3).
    pub fn next(&self) -> Option<NngStatChild<'root>> {
        unsafe {
            let node = self.nng_stat();
            let node = nng_stat_next(node);
            NngStatChild::new(node)
        }
    }
}

impl<'root> NngStat for NngStatChild<'_> {
    unsafe fn nng_stat(&self) -> *mut nng_stat {
        self.node
    }
}

/// Iterator over sibling statistics
#[derive(Debug)]
pub struct Iter<'root> {
    node: Option<NngStatChild<'root>>,
}

impl<'root> Iterator for Iter<'root> {
    type Item = NngStatChild<'root>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node.take();
        if let Some(ref node) = next {
            self.node = node.next();
        }
        next
    }
}
