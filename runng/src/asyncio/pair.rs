//! Async pair

use crate::{asyncio::*, msg::NngMsg, *};
use futures::{future::Future, sync::oneshot};

/// Asynchronous context for request socket.
pub struct PairAsyncHandle {
    push: PushAsyncHandle,
    pull: PullAsyncHandle,
}

impl AsyncContext for PairAsyncHandle {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let push = PushAsyncHandle::create(socket.clone())?;
        let pull = PullAsyncHandle::create(socket)?;
        let ctx = Self { push, pull };
        Ok(ctx)
    }
}

impl AsyncPush for PairAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        self.push.send(msg)
    }
}

impl ReadAsync for PairAsyncHandle {
    fn receive(&mut self) -> Box<dyn Future<Item = NngResult<NngMsg>, Error = oneshot::Canceled>> {
        self.pull.receive()
    }
}
