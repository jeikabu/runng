pub mod publish;
pub mod pull;
pub mod reply;
pub mod request;

pub use self::publish::*;
pub use self::pull::*;
pub use self::reply::*;
pub use self::request::*;

use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

fn nng_open<T, O, S>(open_func: O, socket_create_func: S) -> NngResult<T>
    where O: Fn(&mut nng_socket) -> i32,
        S: Fn(NngSocket) -> T
{
    let mut socket = nng_socket { id: 0 };
    let res = open_func(&mut socket);
    NngFail::succeed_then(res, || {
        let socket = NngSocket::new(socket);
        socket_create_func(socket)
    })
}

pub trait AsyncSocket: Socket {
    type ContextType;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>>;
}