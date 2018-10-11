pub mod publish;
pub mod reply;
pub mod request;
pub mod subscribe;

pub use self::publish::*;
pub use self::reply::*;
pub use self::request::*;
pub use self::subscribe::*;

use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

type MsgFutureType = NngResult<NngMsg>;
type MsgPromise = oneshot::Sender<MsgFutureType>;
type MsgFuture = oneshot::Receiver<MsgFutureType>;
type NngReturnPromise = oneshot::Sender<NngReturn>;
type NngReturnFuture = oneshot::Receiver<NngReturn>;


fn open<T, O, S>(open_func: O, socket_create_func: S) -> NngResult<T>
    where O: Fn(&mut nng_socket) -> i32,
        S: Fn(NngSocket) -> T
{
    let mut socket = nng_socket { id: 0 };
    let res = open_func(&mut socket);
    if res == 0 {
        let socket = NngSocket::new(socket);
        Ok(socket_create_func(socket))
    } else {
        Err(NngFail::from_i32(res))
    }
}

trait Context {
    fn new() -> Box<Self>;
    fn init(&mut self, Rc<NngAio>) -> NngResult<()>;
}

fn create_async_context<T: Context>(socket: NngSocket, callback: AioCallback) -> NngResult<Box<T>> {
    let mut ctx = T::new();
    // This mess is needed to convert Box<_> to c_void
    let ctx_ptr = ctx.as_mut() as *mut _ as AioCallbackArg;
    let aio = NngAio::new(socket, callback, ctx_ptr)?;
    let aio = Rc::new(aio);
    (*ctx).init(aio.clone())?;
    Ok(ctx)
}
