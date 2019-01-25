//! Async pair

use crate::{asyncio::*, msg::NngMsg, *};
use futures::sync::{mpsc, oneshot};

/// Asynchronous context for request socket.
pub struct PairStreamHandle {
    push: PushAsyncHandle,
    pull: PullAsyncStream,
}

impl AsyncContext for PairStreamHandle {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let push = PushAsyncHandle::create(socket.clone())?;
        let pull = PullAsyncStream::create(socket)?;
        let ctx = Self { push, pull };
        Ok(ctx)
    }
}

impl AsyncPush for PairStreamHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        self.push.send(msg)
    }
}

impl AsyncPull for PairStreamHandle {
    fn receive(&mut self) -> Option<mpsc::Receiver<NngResult<NngMsg>>> {
        self.pull.receive()
    }
}
