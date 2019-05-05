//! Asynchronous I/O with `nng_aio`.

pub mod aio;
pub mod pair;
pub mod pair_stream;
pub mod pull;
pub mod pull_stream;
pub mod push;
pub mod reply;
pub mod reply_stream;
pub mod request;
pub mod simple;
pub mod stream;

pub use self::aio::*;
pub use self::pair::*;
pub use self::pair_stream::*;
pub use self::pull::*;
pub use self::pull_stream::*;
pub use self::push::*;
pub use self::reply::*;
pub use self::reply_stream::*;
pub use self::request::*;
pub use self::simple::*;
pub use self::stream::*;

use crate::{msg::NngMsg, *};
use futures::{future, future::Future, sync::mpsc, sync::oneshot, Sink};
use log::debug;
use runng_sys::*;
use std::collections::VecDeque;

/// Context for asynchrounous I/O.
pub trait AsyncContext: Sized {
    /// Create a new asynchronous context using specified socket.
    fn new(socket: NngSocket) -> Result<Self>;
}

pub trait AsyncStreamContext: Sized {
    /// Create a new asynchronous context using specified socket.
    fn new(socket: NngSocket, buffer: usize) -> Result<Self>;
    //fn new_unbounded(socket: NngSocket) -> Result<Self>;
}

/// A `Socket` that can be turned into a context for asynchronous I/O.
///
/// # Examples
/// ```
/// use runng::{
///     *,
///     asyncio::*,
///     factory::latest::ProtocolFactory,
/// };
/// fn test() -> runng::Result<()> {
///     let factory = ProtocolFactory::default();
///     let pusher = factory.pusher_open()?.listen("inproc://test")?;
///     let mut push_ctx = pusher.create_async()?;
///     Ok(())
/// }
/// ```
pub trait AsyncSocket: Socket {
    /// The type of aynchronous context produced
    type ContextType: AsyncContext;

    /// Turns the `Socket` into an asynchronous context
    fn create_async(&self) -> Result<Self::ContextType> {
        let socket = self.socket().clone();
        let ctx = Self::ContextType::new(socket)?;
        Ok(ctx)
    }
}

pub trait AsyncStream: Socket {
    /// The type of aynchronous context produced
    type ContextType: AsyncStreamContext;

    /// Turns the `Socket` into an asynchronous context
    fn create_async_stream(&self, buffer: usize) -> Result<Self::ContextType> {
        let socket = self.socket().clone();
        let ctx = Self::ContextType::new(socket, buffer)?;
        Ok(ctx)
    }
}

fn try_signal_complete(sender: &mut mpsc::Sender<Result<NngMsg>>, message: Result<NngMsg>) {
    let res = sender.try_send(message);
    if let Err(err) = res {
        if err.is_disconnected() {
            let message = err.into_inner();
            debug!("mpsc::disconnected {:?}", message);
            sender.close().unwrap();
        } else {
            debug!("mpsc::send failed {}", err);
        }
    }
}

pub type AsyncMsg = Box<dyn Future<Item = Result<NngMsg>, Error = oneshot::Canceled>>;

#[derive(Debug, Default)]
struct WorkQueue {
    waiting: VecDeque<oneshot::Sender<Result<NngMsg>>>,
    ready: VecDeque<Result<NngMsg>>,
}

impl WorkQueue {
    fn push_back(&mut self, message: Result<NngMsg>) {
        if let Some(sender) = self.waiting.pop_front() {
            sender
                .send(message)
                .unwrap_or_else(|err| debug!("Dropping message: {:?}", err));
        } else {
            self.ready.push_back(message);
        }
    }

    fn pop_front(&mut self) -> AsyncMsg {
        // If a value is ready return it immediately.  Otherwise
        if let Some(item) = self.ready.pop_front() {
            Box::new(future::ok(item))
        } else {
            let (sender, receiver) = oneshot::channel();
            self.waiting.push_back(sender);
            Box::new(receiver)
        }
    }
}

trait NngSink: Sink<SinkItem = Result<NngMsg>, SinkError = mpsc::SendError<Result<NngMsg>>> {}
impl<T: Sink<SinkItem = Result<NngMsg>, SinkError = mpsc::SendError<Result<NngMsg>>>> NngSink
    for T
{
}

/// Represents asynchronous I/O operation performed by NngAio handle.
/// All methods will be called from native threads, impls must be thread-safe.
pub trait AioWork {
    fn begin(&self, aio: &NngAio);
    fn finish(&mut self, aio: &NngAio);
}

/// Trait object asynchronous I/O operation
pub type AioWorkRequest = Box<dyn AioWork>;

/// Queue of pending asynchronous I/O operations.
pub trait AioWorkQueue {
    fn push_back(&mut self, obj: AioWorkRequest);
}
