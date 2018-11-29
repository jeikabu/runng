pub mod pub0;
pub mod pull0;
pub mod push0;
pub mod rep0;
pub mod req0;
pub mod sub0;

pub mod publish;
pub mod pull;
pub mod reply;
pub mod request;

pub use self::pub0::*;
pub use self::pull0::*;
pub use self::push0::*;
pub use self::rep0::*;
pub use self::req0::*;
pub use self::sub0::*;

pub use self::publish::*;
pub use self::pull::*;
pub use self::reply::*;
pub use self::request::*;

use futures::{
    Sink,
    sync::mpsc,
};

use msg::NngMsg;
use runng_sys::*;
use super::*;

pub trait AsyncSocket: Socket {
    type ContextType: AsyncContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>>
    {
        let ctx = Self::ContextType::new(self.take());
        let mut ctx = Box::new(ctx);
        // This mess is needed to convert Box<_> to c_void
        let arg = ctx.as_mut() as *mut _ as AioCallbackArg;
        let res = ctx.as_mut().aio_mut().init(Self::ContextType::get_aio_callback(), arg);
        if let Err(err) = res {
            Err(err)
        } else {
            Ok(ctx)
        }
    }
}

pub trait AsyncContext: Aio {
    fn new(socket: NngSocket) -> Self;
    fn get_aio_callback() -> AioCallback;
}

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

fn try_signal_complete(sender: &mut Option<mpsc::Sender<NngResult<NngMsg>>>, message: NngResult<NngMsg>) {
    if let Some(ref mut sender) = sender {
        let res = sender.try_send(message);
        if let Err(err) = res {
            if err.is_disconnected() {
                sender.close();
            } else {
                debug!("Send failed: {}", err);
            }
        }
    }
}

fn subscribe(socket: nng_socket, topic: &[u8]) -> NngReturn {
    unsafe {
        let opt = NNG_OPT_SUB_SUBSCRIBE.as_ptr() as *const ::std::os::raw::c_char;
        let topic_ptr = topic.as_ptr() as *const ::std::os::raw::c_void;
        let topic_size = std::mem::size_of_val(topic);
        let res = nng_setopt(socket, opt, topic_ptr, topic_size);
        NngFail::from_i32(res)
    }
}
