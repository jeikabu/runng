//! NNG protocols.  See [Section 7](https://nanomsg.github.io/nng/man/v1.1.0/index.html#_section_7_protocols_and_transports).

pub mod pair;
pub mod pair_stream;
pub mod pull;
pub mod pull_stream;
pub mod push;
pub mod reply;
pub mod reply_stream;
pub mod request;

pub use self::pair::*;
pub use self::pair_stream::*;
pub use self::pull::*;
pub use self::pull_stream::*;
pub use self::push::*;
pub use self::reply::*;
pub use self::reply_stream::*;
pub use self::request::*;

use futures::{sync::mpsc, Sink};

use crate::{msg::NngMsg, *};
use futures::{future, future::Future, sync::oneshot};
use std::collections::VecDeque;

/// Context for asynchrounous I/O.
pub trait AsyncContext: Sized {
    /// Create a new asynchronous context using specified socket.
    fn create(socket: NngSocket) -> Result<Self>;
}

pub trait AsyncStreamContext: Sized {
    /// Create a new asynchronous context using specified socket.
    fn create(socket: NngSocket, buffer: usize) -> Result<Self>;
    //fn create_unbounded(socket: NngSocket) -> Result<Self>;
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
        let ctx = Self::ContextType::create(socket)?;
        Ok(ctx)
    }
}

pub trait AsyncStream: Socket {
    /// The type of aynchronous context produced
    type ContextType: AsyncStreamContext;

    /// Turns the `Socket` into an asynchronous context
    fn create_async_stream(&self, buffer: usize) -> Result<Self::ContextType> {
        let socket = self.socket().clone();
        let ctx = Self::ContextType::create(socket, buffer)?;
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

#[derive(Debug, Default)]
struct WorkQueue {
    waiting: VecDeque<oneshot::Sender<Result<NngMsg>>>,
    ready: VecDeque<Result<NngMsg>>,
}

impl WorkQueue {
    fn push_back(&mut self, message: Result<NngMsg>) {
        if let Some(sender) = self.waiting.pop_front() {
            sender.send(message).unwrap();
        } else {
            self.ready.push_back(message);
        }
    }
}

trait NngSink: Sink<SinkItem = Result<NngMsg>, SinkError = mpsc::SendError<Result<NngMsg>>> {}
impl<T: Sink<SinkItem = Result<NngMsg>, SinkError = mpsc::SendError<Result<NngMsg>>>> NngSink
    for T
{
}
