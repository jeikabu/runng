//! Async pair

use super::*;

/// Async pair context for pair protocol.
#[derive(Debug)]
pub struct PairAsyncHandle {
    push: PushAsyncHandle,
    pull: PullAsyncHandle,
}

impl AsyncContext for PairAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let push = PushAsyncHandle::new(socket.clone())?;
        let pull = PullAsyncHandle::new(socket)?;
        let ctx = Self { push, pull };
        Ok(ctx)
    }
}

impl AsyncPush for PairAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> AsyncUnit {
        self.push.send(msg)
    }
}

impl ReadAsync for PairAsyncHandle {
    fn receive(&mut self) -> AsyncMsg {
        self.pull.receive()
    }
}
