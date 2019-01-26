//! Async request/reply

use crate::{
    aio::{AioCallbackArg, NngAio},
    asyncio::*,
    ctx::NngCtx,
    msg::NngMsg,
    *,
};
use futures::sync::oneshot;
use log::{debug, info};
use runng_sys::*;

#[derive(Debug, PartialEq)]
enum RequestState {
    Ready,
    Sending,
    Receiving,
}

struct RequestContextAioArg {
    ctx: NngCtx,
    state: RequestState,
    sender: Option<oneshot::Sender<NngResult<NngMsg>>>,
}

impl RequestContextAioArg {
    pub fn create(socket: NngSocket) -> NngResult<Box<Self>> {
        let ctx = NngCtx::create(socket)?;
        let arg = Self {
            ctx,
            state: RequestState::Ready,
            sender: None,
        };
        NngAio::register_aio(arg, request_callback)
    }
    pub fn send(&mut self, msg: NngMsg, sender: oneshot::Sender<NngResult<NngMsg>>) {
        if self.state != RequestState::Ready {
            panic!();
        }
        self.sender = Some(sender);
        unsafe {
            let aio = self.ctx.aio().nng_aio();
            let ctx = self.ctx.ctx();
            self.state = RequestState::Sending;

            // Nng assumes ownership of the message
            let msg = msg.take();
            nng_aio_set_msg(aio, msg);
            nng_ctx_send(ctx, aio);
        }
    }
}

impl Aio for RequestContextAioArg {
    fn aio(&self) -> &NngAio {
        self.ctx.aio()
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        self.ctx.aio_mut()
    }
}

/// Asynchronous context for request socket.
pub struct RequestAsyncHandle {
    aio_arg: Box<RequestContextAioArg>,
}

impl AsyncContext for RequestAsyncHandle {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let aio_arg = RequestContextAioArg::create(socket)?;
        let ctx = Self { aio_arg };
        Ok(ctx)
    }
}

/// Trait for asynchronous contexts that can send a request and receive a reply.
pub trait AsyncRequest {
    /// Asynchronously send a request and return a future for the reply.
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngResult<NngMsg>>;
}

impl AsyncRequest for RequestAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngResult<NngMsg>> {
        let (sender, receiver) = oneshot::channel::<NngResult<NngMsg>>();
        self.aio_arg.send(msg, sender);
        receiver
    }
}

unsafe extern "C" fn request_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut RequestContextAioArg);
    let aionng = ctx.ctx.aio().nng_aio();
    let ctxnng = ctx.ctx.ctx();
    trace!("callback Request:{:?}", ctx.state);
    match ctx.state {
        RequestState::Ready => panic!(),
        RequestState::Sending => {
            let res = NngFail::from_i32(nng_aio_result(aionng));
            match res {
                Err(res) => {
                    // Nng requries we resume ownership of the message
                    let _ = NngMsg::new_msg(nng_aio_get_msg(aionng));

                    ctx.state = RequestState::Ready;
                    let sender = ctx.sender.take().unwrap();
                    sender.send(Err(res)).unwrap();
                }
                Ok(()) => {
                    ctx.state = RequestState::Receiving;
                    nng_ctx_recv(ctxnng, aionng);
                }
            }
        }
        RequestState::Receiving => {
            let sender = ctx.sender.take().unwrap();
            let res = NngFail::from_i32(nng_aio_result(aionng));
            match res {
                Err(res) => {
                    ctx.state = RequestState::Ready;
                    let res = sender.send(Err(res));
                    if let Err(res) = res {
                        debug!("Receive failed to send error: {:?}", res);
                    }
                }
                Ok(()) => {
                    let msg = NngMsg::new_msg(nng_aio_get_msg(aionng));

                    ctx.state = RequestState::Ready;
                    let res = sender.send(Ok(msg));
                    if let Err(msg) = res {
                        info!("Dropping request: {:?}", msg);
                    }
                }
            }
        }
    }
}
