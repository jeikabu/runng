/*!
Runtime statistics

Statistics are organized as a tree.  Starting at the root, nodes may have a child whose siblings are likewise children of the parent.

## Examples

```rust
use log::{debug};
use runng::{
    *,
    stats::NngStat,
    stats::NngStatRoot,
};

#[test]
fn stats_example() -> NngReturn {
    // https://github.com/nanomsg/nng/issues/841
    let url = "inproc://test";
    let factory = Latest::default();
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

use runng_sys::*;
use crate::*;

/// Type of statistic.  See `NngStatChild::stat_type`.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum NngStatType {
    Scope  = nng_stat_type_enum_NNG_STAT_SCOPE,
    Level  = nng_stat_type_enum_NNG_STAT_LEVEL,
    Counter    = nng_stat_type_enum_NNG_STAT_COUNTER,
    String     = nng_stat_type_enum_NNG_STAT_STRING,
    Boolean    = nng_stat_type_enum_NNG_STAT_BOOLEAN,
    Id     = nng_stat_type_enum_NNG_STAT_ID,
}

impl NngStatType {
    /// Converts value returned by [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3) into `NngStatType`.
    pub fn from_i32(value: i32) -> Option<NngStatType> {
        match value {
            value if value == NngStatType::Scope as i32    => Some(NngStatType::Scope),
            value if value == NngStatType::Level as i32    => Some(NngStatType::Level),
            value if value == NngStatType::Counter as i32  => Some(NngStatType::Counter),
            value if value == NngStatType::String as i32   => Some(NngStatType::String),
            value if value == NngStatType::Boolean as i32  => Some(NngStatType::Boolean),
            value if value == NngStatType::Id as i32   => Some(NngStatType::Id),
            _   => None,
        }
    }
}

/// Unit of quantity measured by statistic.  See `NngStatChild::unit()`.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum NngStatUnit {
    None   = nng_unit_enum_NNG_UNIT_NONE,
    Bytes  = nng_unit_enum_NNG_UNIT_BYTES,
    Messages   = nng_unit_enum_NNG_UNIT_MESSAGES,
    Millis = nng_unit_enum_NNG_UNIT_MILLIS,
    Events = nng_unit_enum_NNG_UNIT_EVENTS,
}

impl NngStatUnit {
    /// Converts value returned by [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3) into `NngStatUnit`.
    pub fn from_i32(value: i32) -> Option<NngStatUnit> {
        match value {
            value if value == NngStatUnit::None as i32  => Some(NngStatUnit::None),
            value if value == NngStatUnit::Bytes as i32 => Some(NngStatUnit::Bytes),
            value if value == NngStatUnit::Messages as i32  => Some(NngStatUnit::Messages),
            value if value == NngStatUnit::Millis as i32    => Some(NngStatUnit::Millis),
            value if value == NngStatUnit::Events as i32    => Some(NngStatUnit::Events),
            _   => None,
        }
    }
}

pub trait NngStat {
    /// Obtain underlying [`nng_stat`](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat.5).
    unsafe fn nng_stat(&self) -> *mut nng_stat;
    /// Returns the first child statistic, if any.
    fn child(&self) -> Option<NngStatChild> {
        unsafe {
            let node = nng_stat_child(self.nng_stat());
            if node.is_null() {
                None
            } else {
                Some(NngStatChild{ node })
            }
        }
    }
}

/// Root of tree of statistics snapshot.
/// 
/// ## Examples
/// ```rust,no_run
/// use runng::{stats::NngStat, stats::NngStatRoot};
/// let child = NngStatRoot::new().unwrap().child();
/// ```
pub struct NngStatRoot {
    node: *mut nng_stat,
}

impl NngStatRoot {
    /// Get statistics snapshot.  See [nng_stats_get](https://nanomsg.github.io/nng/man/v1.1.0/nng_stats_get.3).
    pub fn new() -> NngResult<NngStatRoot> {
        unsafe {
            let mut node: *mut nng_stat = std::ptr::null_mut();
            let res = nng_stats_get(&mut node);
            NngFail::succeed_then(res, || NngStatRoot{ node })
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
            nng_stats_free(self.node)
        }
    }
}

pub struct NngStatChild {
    node: *mut nng_stat,
}

/// Child of statistic node in tree of statistics.  See `NngStat::child()`.
impl NngStatChild {
    /// See [nng_stat_name](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_name.3).
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_name(self.nng_stat());
            std::ffi::CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_desc](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_desc.3).
    pub fn desc(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_desc(self.nng_stat());
            std::ffi::CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3).
    pub fn stat_type(&self) -> Option<NngStatType> {
        unsafe {
            let val = nng_stat_type(self.nng_stat());
            NngStatType::from_i32(val)
        }
    }

    /// See [nng_stat_value](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn value(&self) -> u64 {
        unsafe {
            nng_stat_value(self.nng_stat())
        }
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
            let string = std::ffi::CStr::from_ptr(ptr).to_str();
            if let Ok(string) = string {
                Some(string)
            } else {
                None
            }
        }
    }

    /// See [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3).
    pub fn unit(&self) -> Option<NngStatUnit> {
        unsafe {
            let val = nng_stat_unit(self.nng_stat());
            NngStatUnit::from_i32(val)
        }
    }

    /// See [nng_stat_timestamp](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn timestamp(&self) -> u64 {
        unsafe {
            nng_stat_timestamp(self.nng_stat())
        }
    }

    /// Returns an iterator over sibling statistics.  See [nng_stat_next](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_next.3).
    pub fn iter(&self) -> Iter {
        Iter { node: self.node }
    }
}

impl NngStat for NngStatChild {
    unsafe fn nng_stat(&self) -> *mut nng_stat {
        self.node
    }
}

/// Iterator over sibling statistics
pub struct Iter {
    node: *mut nng_stat,
}

impl Iterator for Iter {
    type Item = NngStatChild;
    fn next(&mut self) -> Option<Self::Item> {
        if self.node.is_null() {
            return None;
        }
        unsafe {   
            self.node = nng_stat_next(self.node);
        }
        if self.node.is_null() {
            None
        } else {
            Some(NngStatChild{node: self.node})
        }
    }
}