//! Async pair

use crate::{msg::NngMsg, protocol::*, *};
use futures::sync::{mpsc, oneshot};

/// Asynchronous context for request socket.
pub struct AsyncPairContext {
    push_aio_arg: Box<PushContextAioArg>,
    pull_aio_arg: Box<PullContextAioArg>,
    // Input queue is synchronous
    //in_sender: Wait<mpsc::Sender<SendRequest>>,
    receiver: Option<mpsc::Receiver<NngResult<NngMsg>>>,
}

impl AsyncContext for AsyncPairContext {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let push_aio_arg = PushContextAioArg::create(socket.clone())?;
        let (sender, receiver) = mpsc::channel::<NngResult<NngMsg>>(1024);
        let pull_aio_arg = PullContextAioArg::create(socket, sender)?;
        //let in_sender = in_sender.wait(); // Make input queue synchronous
        let receiver = Some(receiver);
        let ctx = Self {
            push_aio_arg,
            pull_aio_arg,
            receiver,
        };
        Ok(ctx)
    }
}

impl AsyncPush for AsyncPairContext {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        let (sender, receiver) = oneshot::channel::<NngReturn>();
        self.push_aio_arg.send(msg, sender);
        receiver
    }
}

impl AsyncPull for AsyncPairContext {
    fn receive(&mut self) -> Option<mpsc::Receiver<NngResult<NngMsg>>> {
        let receiver = self.receiver.take();
        if receiver.is_some() {
            self.pull_aio_arg.start_receive();
        }
        receiver
    }
}
