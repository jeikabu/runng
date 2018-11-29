//! Async push/pull ("pipeline")

use aio::{NngAio, AioCallbackArg};
use futures::{
    sync::mpsc::{
        channel,
        Receiver,
        Sender,
    }
    };
use msg::NngMsg;
use runng_sys::*;
use super::*;

#[derive(Debug,PartialEq)]
enum PullState {
    Ready,
    Receiving,
}

/// Asynchronous context for pull socket.
pub struct AsyncPullContext {
    aio: NngAio,
    state: PullState,
    sender: Option<Sender<NngResult<NngMsg>>>,
}

impl AsyncPullContext {
    fn start_receive(&mut self) {
        self.state = PullState::Receiving;
        unsafe {
            nng_recv_aio(self.aio.nng_socket(), self.aio.nng_aio());
        }
    }
}

impl AsyncContext for AsyncPullContext {
    fn new(socket: NngSocket) -> NngResult<Self> {
        let aio = NngAio::new(socket);
        let ctx = Self {
            aio,
            state: PullState::Ready,
            sender: None,
        };
        Ok(ctx)
    }
    fn get_aio_callback() -> AioCallback {
        pull_callback
    }
}

impl Aio for AsyncPullContext {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Trait for asynchronous contexts that can receive a stream of messages.
pub trait AsyncPull {
    /// Asynchronously receive a stream of messages.
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>>;
}

impl AsyncPull for AsyncPullContext {
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>> {
        let (sender, receiver) = channel::<NngResult<NngMsg>>(1024);
        self.sender = Some(sender);
        self.start_receive();
        receiver
    }
}

extern fn pull_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPullContext);
        
        trace!("callback Subscribe:{:?}", ctx.state);
        match ctx.state {
            PullState::Ready => panic!(),
            PullState::Receiving => {
                let aio = ctx.aio.nng_aio();
                let res = NngFail::from_i32(nng_aio_result(aio));
                match res {
                    Err(res) => {
                        match res {
                            NngFail::Err(NngError::ECLOSED) => {
                                debug!("Closed");
                            },
                            _ => {
                                trace!("Reply.Receive: {:?}", res);
                                ctx.start_receive();
                            },
                        }
                        try_signal_complete(&mut ctx.sender, Err(res));
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
                        // Make sure to reset state before signaling completion.  Otherwise
                        // have race-condition where receiver can receive None promise
                        ctx.start_receive();
                        try_signal_complete(&mut ctx.sender, Ok(msg));
                    }
                }
            },
        }
    }
}

/// Asynchronous context for subscribe socket.
pub struct AsyncSubscribeContext {
    ctx: AsyncPullContext,
}

impl AsyncPull for AsyncSubscribeContext {
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>> {
        let (sender, receiver) = channel::<NngResult<NngMsg>>(1024);
        self.ctx.sender = Some(sender);
        self.ctx.start_receive();
        receiver
    }
}

impl AsyncContext for AsyncSubscribeContext {
    /// Create an asynchronous context using the specified socket.
    fn new(socket: NngSocket) -> NngResult<Self> {
        let aio = NngAio::new(socket);
        let ctx = Self {
            ctx: AsyncPullContext {
                aio,
                state: PullState::Ready,
                sender: None,
            }
        };
        Ok(ctx)
    }
    fn get_aio_callback() -> AioCallback {
        pull_callback
    }
}

impl Aio for AsyncSubscribeContext {
    fn aio(&self) -> &NngAio {
        &self.ctx.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.ctx.aio
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
        unsafe {
            subscribe(self.ctx.aio.nng_socket(), topic)
        }
    }
}
