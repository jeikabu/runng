//! Async bus

use super::*;

/// Async pair context for pair protocol.
#[derive(Debug)]
pub struct BusAsyncHandle {
    push: PushAsyncHandle,
    pull: PullAsyncHandle,
}

impl AsyncContext for BusAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let push = PushAsyncHandle::new(socket.clone())?;
        let pull = PullAsyncHandle::new(socket)?;
        let ctx = Self { push, pull };
        Ok(ctx)
    }
}

impl AsyncPush for BusAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>> {
        self.push.send(msg)
    }
}

impl ReadAsync for BusAsyncHandle {
    fn receive(&mut self) -> AsyncMsg {
        self.pull.receive()
    }
}
