//! NNG protocols.  See [Section 7](https://nanomsg.github.io/nng/man/v1.1.0/index.html#_section_7_protocols_and_transports).

pub mod pair0;
pub mod pair1;
pub mod pub0;
pub mod pull0;
pub mod push0;
pub mod rep0;
pub mod req0;
pub mod sub0;

pub use self::pair0::*;
pub use self::pair1::*;
pub use self::pub0::*;
pub use self::pull0::*;
pub use self::push0::*;
pub use self::rep0::*;
pub use self::req0::*;
pub use self::sub0::*;

use crate::{msg::NngMsg, *};
use runng_sys::*;
use runng_sys::*;
use std::{collections::VecDeque, sync::Mutex};

pub trait Subscribe {
    fn subscribe(&self, topic: &[u8]) -> NngReturn;
    fn subscribe_str(&self, topic: &str) -> NngReturn {
        self.subscribe(topic.as_bytes())
    }
}

fn nng_open<T, O, S>(open_func: O, socket_create_func: S) -> NngResult<T>
where
    O: Fn(&mut nng_socket) -> i32,
    S: Fn(NngSocket) -> T,
{
    let mut socket = nng_socket { id: 0 };
    let res = open_func(&mut socket);
    NngFail::succeed_then(res, || {
        let socket = NngSocket::new(socket);
        socket_create_func(socket)
    })
}

pub(crate) fn subscribe(socket: nng_socket, topic: &[u8]) -> NngReturn {
    unsafe {
        let opt = NNG_OPT_SUB_SUBSCRIBE.as_ptr() as *const ::std::os::raw::c_char;
        let topic_ptr = topic.as_ptr() as *const ::std::os::raw::c_void;
        let topic_size = std::mem::size_of_val(topic);
        let res = nng_setopt(socket, opt, topic_ptr, topic_size);
        NngFail::from_i32(res)
    }
}
