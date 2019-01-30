//! Async push/pull ("pipeline")

use crate::{
    aio::{AioCallbackArg, NngAio},
    asyncio::*,
    msg::NngMsg,
    protocol::{subscribe, Subscribe},
    *,
};
use futures::sync::mpsc;
use runng_sys::*;

#[derive(Debug, PartialEq)]
enum PullState {
    Ready,
    Receiving,
}

struct PullContextAioArg {
    aio: NngAio,
    state: PullState,
    sender: mpsc::Sender<NngResult<NngMsg>>,
}

impl PullContextAioArg {
    pub fn create(
        socket: NngSocket,
        sender: mpsc::Sender<NngResult<NngMsg>>,
    ) -> NngResult<Box<Self>> {
        let aio = NngAio::new(socket);
        let arg = Self {
            aio,
            state: PullState::Ready,
            sender,
        };
        NngAio::register_aio(arg, pull_callback)
    }

    pub(crate) fn start_receive(&mut self) {
        self.state = PullState::Receiving;
        unsafe {
            nng_recv_aio(self.aio.nng_socket(), self.aio.nng_aio());
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
pub struct PullAsyncStream {
    aio_arg: Box<PullContextAioArg>,
    receiver: Option<mpsc::Receiver<NngResult<NngMsg>>>,
}

impl AsyncStreamContext for PullAsyncStream {
    fn create(socket: NngSocket, buffer: usize) -> NngResult<Self> {
        let (sender, receiver) = mpsc::channel::<NngResult<NngMsg>>(buffer);
        let aio_arg = PullContextAioArg::create(socket, sender)?;
        let receiver = Some(receiver);
        Ok(Self { aio_arg, receiver })
    }
}

/// Trait for asynchronous contexts that can receive a stream of messages.
pub trait AsyncPull {
    /// Asynchronously receive a stream of messages.
    fn receive(&mut self) -> Option<mpsc::Receiver<NngResult<NngMsg>>>;
}

impl AsyncPull for PullAsyncStream {
    fn receive(&mut self) -> Option<mpsc::Receiver<NngResult<NngMsg>>> {
        let receiver = self.receiver.take();
        if receiver.is_some() {
            self.aio_arg.start_receive();
        }
        receiver
    }
}

unsafe extern "C" fn pull_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut PullContextAioArg);
    trace!("pull_callback::{:?}", ctx.state);
    match ctx.state {
        PullState::Ready => panic!(),
        PullState::Receiving => {
            let aio = ctx.aio.nng_aio();
            let aio_res = nng_aio_result(aio);
            let res = NngFail::from_i32(aio_res);
            match res {
                Err(res) => {
                    match res {
                        // nng_aio_close() calls nng_aio_stop which nng_aio_abort(NNG_ECANCELED) and waits.
                        // If we call start_receive() it will fail with ECANCELED and we infinite loop...
                        NngFail::Err(NngError::ECLOSED) | NngFail::Err(NngError::ECANCELED) => {
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
                    let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
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
pub struct SubscribeAsyncHandle {
    ctx: PullAsyncStream,
}

impl AsyncPull for SubscribeAsyncHandle {
    fn receive(&mut self) -> Option<mpsc::Receiver<NngResult<NngMsg>>> {
        self.ctx.receive()
    }
}

impl AsyncStreamContext for SubscribeAsyncHandle {
    /// Create an asynchronous context using the specified socket.
    fn create(socket: NngSocket, buffer: usize) -> NngResult<Self> {
        let ctx = PullAsyncStream::create(socket, buffer)?;
        let ctx = Self { ctx };
        Ok(ctx)
    }
}

impl InternalSocket for SubscribeAsyncHandle {
    fn socket(&self) -> &NngSocket {
        self.ctx.aio_arg.aio().socket()
    }
}

impl Subscribe for SubscribeAsyncHandle {
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe { subscribe(self.socket().nng_socket(), topic) }
    }
}
