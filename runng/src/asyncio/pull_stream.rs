//! Async push/pull ("pipeline")

use crate::{asyncio::*, msg::NngMsg, protocol::*, *};
use futures::sync::mpsc;
use runng_sys::*;

#[derive(Debug, PartialEq)]
enum PullState {
    Ready,
    Receiving,
}

#[derive(Debug)]
struct PullContextAioArg {
    aio: NngAio,
    state: PullState,
    sender: mpsc::Sender<Result<NngMsg>>,
    socket: NngSocket,
}

impl PullContextAioArg {
    pub fn new(socket: NngSocket, sender: mpsc::Sender<Result<NngMsg>>) -> Result<AioArg<Self>> {
        NngAio::new(
            |aio| Self {
                aio,
                state: PullState::Ready,
                sender,
                socket,
            },
            pull_callback,
        )
    }

    pub(crate) fn start_receive(&mut self) {
        self.state = PullState::Receiving;
        unsafe {
            nng_recv_aio(self.socket.nng_socket(), self.aio.nng_aio());
        }
    }
}

impl Aio for PullContextAioArg {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Asynchronous context for pull socket.
#[derive(Debug)]
pub struct PullAsyncStream {
    aio_arg: AioArg<PullContextAioArg>,
    receiver: Option<mpsc::Receiver<Result<NngMsg>>>,
}

impl AsyncStreamContext for PullAsyncStream {
    fn new(socket: NngSocket, buffer: usize) -> Result<Self> {
        let (sender, receiver) = mpsc::channel::<Result<NngMsg>>(buffer);
        let aio_arg = PullContextAioArg::new(socket, sender)?;
        let receiver = Some(receiver);
        Ok(Self { aio_arg, receiver })
    }
}

/// Trait for asynchronous contexts that can receive a stream of messages.
pub trait AsyncPull {
    /// Asynchronously receive a stream of messages.
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>>;
}

impl AsyncPull for PullAsyncStream {
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>> {
        let receiver = self.receiver.take();
        if receiver.is_some() {
            self.aio_arg.start_receive();
        }
        receiver
    }
}

unsafe extern "C" fn pull_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut PullContextAioArg);
    trace!("pull_callback::{:?}", ctx.state);
    match ctx.state {
        PullState::Ready => panic!(),
        PullState::Receiving => {
            let aio = ctx.aio.nng_aio();
            let aio_res = nng_aio_result(aio);
            let res = nng_int_to_result(aio_res);
            match res {
                Err(res) => {
                    match res {
                        // nng_aio_close() calls nng_aio_stop which nng_aio_abort(NNG_ECANCELED) and waits.
                        // If we call start_receive() it will fail with ECANCELED and we infinite loop...
                        Error::Errno(NngErrno::ECLOSED) | Error::Errno(NngErrno::ECANCELED) => {
                            debug!("pull_callback {:?}", res);
                        }
                        _ => {
                            trace!("pull_callback::Err({:?})", res);
                            ctx.start_receive();
                        }
                    }
                    try_signal_complete(&mut ctx.sender, Err(res));
                }
                Ok(()) => {
                    let msg = NngMsg::from_raw(nng_aio_get_msg(aio));
                    // Make sure to reset state before signaling completion.  Otherwise
                    // have race-condition where receiver can receive None promise
                    ctx.start_receive();
                    try_signal_complete(&mut ctx.sender, Ok(msg));
                }
            }
        }
    }
}

/// Asynchronous context for subscribe socket.
#[derive(Debug)]
pub struct SubscribeAsyncHandle {
    ctx: PullAsyncStream,
}

impl AsyncPull for SubscribeAsyncHandle {
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>> {
        self.ctx.receive()
    }
}

impl AsyncStreamContext for SubscribeAsyncHandle {
    /// Create an asynchronous context using the specified socket.
    fn new(socket: NngSocket, buffer: usize) -> Result<Self> {
        let ctx = PullAsyncStream::new(socket, buffer)?;
        let ctx = Self { ctx };
        Ok(ctx)
    }
}

impl InternalSocket for SubscribeAsyncHandle {
    fn socket(&self) -> &NngSocket {
        &self.ctx.aio_arg.socket
    }
}

impl Subscribe for SubscribeAsyncHandle {
    fn subscribe(&self, topic: &[u8]) -> Result<()> {
        unsafe { subscribe(self.socket().nng_socket(), topic) }
    }
    fn unsubscribe(&self, topic: &[u8]) -> Result<()> {
        unsafe { unsubscribe(self.socket().nng_socket(), topic) }
    }
}
