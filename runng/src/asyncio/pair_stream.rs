//! Async pair

use super::*;

/// Asynchronous context for request socket.
#[derive(Debug)]
pub struct PairStreamHandle {
    push: PushAsyncHandle,
    pull: PullAsyncStream,
}

impl AsyncStreamContext for PairStreamHandle {
    fn new(socket: NngSocket, buffer: usize) -> Result<Self> {
        let push = PushAsyncHandle::new(socket.clone())?;
        let pull = PullAsyncStream::new(socket, buffer)?;
        let ctx = Self { push, pull };
        Ok(ctx)
    }
}

impl AsyncPush for PairStreamHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>> {
        self.push.send(msg)
    }
}

impl AsyncPull for PairStreamHandle {
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>> {
        self.pull.receive()
    }
}
