//! Async request/reply

use crate::{
    aio::{AioCallbackArg, NngAio},
    ctx::NngCtx,
    msg::NngMsg,
    protocol::AsyncContext,
    *,
};
use futures::sync::oneshot::{channel, Receiver, Sender};
use runng_sys::*;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
enum RequestState {
    Ready,
    Sending,
    Receiving,
}

/// Asynchronous context for request socket.
pub struct AsyncRequestContext {
    ctx: NngCtx,
    state: RequestState,
    sender: Option<Sender<NngResult<NngMsg>>>,
}

impl AsyncContext for AsyncRequestContext {
    fn new(socket: Arc<NngSocket>) -> NngResult<Self> {
        let ctx = NngCtx::new(socket)?;
        let ctx = Self {
            ctx,
            state: RequestState::Ready,
            sender: None,
        };
        Ok(ctx)
    }
    fn get_aio_callback() -> AioCallback {
        request_callback
    }
}

impl Aio for AsyncRequestContext {
    fn aio(&self) -> &NngAio {
        self.ctx.aio()
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        self.ctx.aio_mut()
    }
}

/// Trait for asynchronous contexts that can send a request and receive a reply.
pub trait AsyncRequest {
    /// Asynchronously send a request and return a future for the reply.
    fn send(&mut self, msg: NngMsg) -> Receiver<NngResult<NngMsg>>;
}

impl AsyncRequest for AsyncRequestContext {
    fn send(&mut self, msg: NngMsg) -> Receiver<NngResult<NngMsg>> {
        if self.state != RequestState::Ready {
            panic!();
        }
        let (sender, receiver) = channel::<NngResult<NngMsg>>();
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

        receiver
    }
}

unsafe extern "C" fn request_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut AsyncRequestContext);
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
                    sender.send(Err(res)).unwrap();
                }
                Ok(()) => {
                    let msg = NngMsg::new_msg(nng_aio_get_msg(aionng));

                    ctx.state = RequestState::Ready;
                    sender.send(Ok(msg)).unwrap();
                }
            }
        }
    }
}
