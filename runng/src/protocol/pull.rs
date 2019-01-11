//! Async push/pull ("pipeline")

use crate::{
    aio::{AioCallbackArg, NngAio},
    msg::NngMsg,
    protocol::{subscribe, try_signal_complete, AsyncContext},
    *,
};
use futures::sync::mpsc::{channel, Receiver, Sender};
use runng_sys::*;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
enum PullState {
    Ready,
    Receiving,
}

struct PullContextAioArg {
    aio: NngAio,
    state: PullState,
    sender: Sender<NngResult<NngMsg>>,
}

impl PullContextAioArg {
    pub fn create(
        socket: Arc<NngSocket>,
        sender: Sender<NngResult<NngMsg>>,
    ) -> NngResult<Box<Self>> {
        let aio = NngAio::new(socket);
        let arg = Self {
            aio,
            state: PullState::Ready,
            sender,
        };
        NngAio::register_aio(arg, pull_callback)
    }

    fn start_receive(&mut self) {
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
pub struct AsyncPullContext {
    aio_arg: Box<PullContextAioArg>,
    receiver: Option<Receiver<NngResult<NngMsg>>>,
}

impl AsyncContext for AsyncPullContext {
    fn create(socket: Arc<NngSocket>) -> NngResult<Self> {
        let (sender, receiver) = channel::<NngResult<NngMsg>>(1024);
        let aio_arg = PullContextAioArg::create(socket, sender)?;
        let receiver = Some(receiver);
        Ok(Self { aio_arg, receiver })
    }
}

/// Trait for asynchronous contexts that can receive a stream of messages.
pub trait AsyncPull {
    /// Asynchronously receive a stream of messages.
    fn receive(&mut self) -> Option<Receiver<NngResult<NngMsg>>>;
}

impl AsyncPull for AsyncPullContext {
    fn receive(&mut self) -> Option<Receiver<NngResult<NngMsg>>> {
        let receiver = self.receiver.take();
        if receiver.is_some() {
            self.aio_arg.start_receive();
        }
        receiver
    }
}

unsafe extern "C" fn pull_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut PullContextAioArg);
    trace!("callback Subscribe:{:?}", ctx.state);
    match ctx.state {
        PullState::Ready => panic!(),
        PullState::Receiving => {
            let aio = ctx.aio.nng_aio();
            let aio_res = nng_aio_result(aio);
            let res = NngFail::from_i32(aio_res);
            match res {
                Err(res) => {
                    match res {
                        NngFail::Err(NngError::ECLOSED) => {
                            debug!("Closed");
                        }
                        _ => {
                            trace!("Reply.Receive: {:?} {:?}", res, aio);
                            ctx.start_receive();
                        }
                    }
                    try_signal_complete(&mut ctx.sender, Err(res));
                }
                Ok(()) => {
                    let msg = nng_aio_get_msg(aio);
                    //debug!("recv {:?} {:?} {:?}", aio_res, msg, aio);
                    let msg = NngMsg::new_msg(msg);
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
pub struct AsyncSubscribeContext {
    ctx: AsyncPullContext,
}

impl AsyncPull for AsyncSubscribeContext {
    fn receive(&mut self) -> Option<Receiver<NngResult<NngMsg>>> {
        self.ctx.receive()
    }
}

impl AsyncContext for AsyncSubscribeContext {
    /// Create an asynchronous context using the specified socket.
    fn create(socket: Arc<NngSocket>) -> NngResult<Self> {
        let ctx = AsyncPullContext::create(socket)?;
        let ctx = Self { ctx };
        Ok(ctx)
    }
}

impl InternalSocket for AsyncSubscribeContext {
    fn socket(&self) -> &NngSocket {
        self.ctx.aio_arg.aio().socket()
    }
}

pub trait Subscribe {
    fn subscribe(&self, topic: &[u8]) -> NngReturn;
    fn subscribe_str(&self, topic: &str) -> NngReturn {
        self.subscribe(topic.as_bytes())
    }
}

impl Subscribe for AsyncSubscribeContext {
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe { subscribe(self.socket().nng_socket(), topic) }
    }
}
