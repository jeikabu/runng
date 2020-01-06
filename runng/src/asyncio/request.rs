//! Async request/reply

use super::*;
use crate::ctx::NngCtx;
use log::{debug, info};

#[derive(Debug, PartialEq)]
enum RequestState {
    Ready,
    Sending,
    Receiving,
}

#[derive(Debug)]
struct RequestContextAioArg {
    aio: NngAio,
    ctx: NngCtx,
    sender: Option<oneshot::Sender<Result<NngMsg>>>,
    socket: NngSocket,
    state: RequestState,
}

impl RequestContextAioArg {
    pub fn new(socket: NngSocket) -> Result<AioArg<Self>> {
        let ctx = NngCtx::new(socket.clone())?;
        NngAio::create(
            |aio| Self {
                aio,
                ctx,
                sender: None,
                socket,
                state: RequestState::Ready,
            },
            request_callback,
        )
    }
    pub fn send(&mut self, msg: NngMsg, sender: oneshot::Sender<Result<NngMsg>>) {
        if self.state != RequestState::Ready {
            panic!();
        }
        self.sender = Some(sender);
        unsafe {
            let aio = self.aio.nng_aio();
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
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Async request context for request/reply pattern.
#[derive(Debug)]
pub struct RequestAsyncHandle {
    aio_arg: AioArg<RequestContextAioArg>,
}

impl AsyncContext for RequestAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let aio_arg = RequestContextAioArg::new(socket)?;
        let ctx = Self { aio_arg };
        Ok(ctx)
    }
}

/// Trait for async contexts that can send a request and receive a reply.
pub trait AsyncRequest {
    /// Asynchronously send a request and return a future for the reply.
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<NngMsg>>;
}

impl AsyncRequest for RequestAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<NngMsg>> {
        let (sender, receiver) = oneshot::channel::<Result<NngMsg>>();
        self.aio_arg.send(msg, sender);
        receiver
    }
}

unsafe extern "C" fn request_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut RequestContextAioArg);
    let aionng = ctx.aio.nng_aio();
    let ctxnng = ctx.ctx.ctx();
    trace!("callback Request:{:?}", ctx.state);
    match ctx.state {
        RequestState::Ready => panic!(),
        RequestState::Sending => {
            let res = nng_int_to_result(nng_aio_result(aionng));
            match res {
                Err(res) => {
                    // Nng requries we resume ownership of the message
                    let _ = NngMsg::from_raw(nng_aio_get_msg(aionng));

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
            let res = nng_int_to_result(nng_aio_result(aionng));
            match res {
                Err(res) => {
                    ctx.state = RequestState::Ready;
                    let res = sender.send(Err(res));
                    if let Err(res) = res {
                        debug!("Receive failed to send error: {:?}", res);
                    }
                }
                Ok(()) => {
                    let msg = NngMsg::from_raw(nng_aio_get_msg(aionng));

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
