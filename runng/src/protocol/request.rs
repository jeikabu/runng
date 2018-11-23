use aio::{NngAio, AioCallbackArg};
use ctx::NngCtx;
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

pub struct Req0 {
    socket: NngSocket
}

impl Req0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0{ socket };
        nng_open(open_func, socket_create_func)
    }
}

#[derive(Debug,PartialEq)]
enum RequestState {
    Ready,
    Sending,
    Receiving,
}

pub trait AsyncRequest {
    fn send(&mut self, msg: NngMsg) -> MsgFuture;
}

pub struct AsyncRequestContext {
    ctx: Option<NngCtx>,
    state: RequestState,
    sender: Option<MsgPromise>
}

impl Context for AsyncRequestContext {
    fn new() -> Box<AsyncRequestContext> {
        let ctx = AsyncRequestContext {
            ctx: None,
            state: RequestState::Ready,
            sender: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngReturn {
        let ctx = NngCtx::new(aio)?;
        self.ctx = Some(ctx);
        Ok(())
    }
}

impl AsyncRequest for AsyncRequestContext {
    fn send(&mut self, msg: NngMsg) -> MsgFuture {
        if self.state != RequestState::Ready {
            panic!();
        }
        let (sender, receiver) = oneshot::channel::<MsgFutureType>();
        self.sender = Some(sender);
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();
            self.state = RequestState::Sending;

            // Nng assumes ownership of the message
            let msg = msg.take();
            nng_aio_set_msg(aio, msg);
            nng_ctx_send(ctx, aio);
        }
        
        receiver
    }
}

impl Socket for Req0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Req0 {}
impl SendMsg for Req0 {}

impl AsyncSocket for Req0 {
    type ContextType = AsyncRequestContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, request_callback)
    }
}

extern fn request_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncRequestContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
        println!("callback Request:{:?}", ctx.state);
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
                    },
                    Ok(()) => {
                        ctx.state = RequestState::Receiving;
                        nng_ctx_recv(ctxnng, aionng);
                    },
                }
            },
            RequestState::Receiving => {
                let sender = ctx.sender.take().unwrap();
                let res = NngFail::from_i32(nng_aio_result(aionng));
                match res {
                    Err(res) => {
                        ctx.state = RequestState::Ready;
                        sender.send(Err(res)).unwrap();
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aionng));
                        
                        ctx.state = RequestState::Ready;
                        sender.send(Ok(msg)).unwrap();
                    },
                }
            },
        }
    }
}
