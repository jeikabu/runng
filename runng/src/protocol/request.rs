use aio::{NngAio, AioCallbackArg};
use ctx::NngCtx;
use futures::{
    sync::oneshot::{
        channel,
        Receiver,
        Sender,
    }
};
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
    fn send(&mut self, msg: NngMsg) -> Receiver<NngResult<NngMsg>>;
}

pub struct AsyncRequestContext {
    ctx: NngCtx,
    state: RequestState,
    sender: Option<Sender<NngResult<NngMsg>>>
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
        let ctx = NngCtx::new(self.socket)?;
        let ctx = Self::ContextType {
            ctx,
            state: RequestState::Ready,
            sender: None,
        };
        
        let mut ctx = Box::new(ctx);
        // This mess is needed to convert Box<_> to c_void
        let arg = ctx.as_mut() as *mut _ as AioCallbackArg;
        let res = ctx.as_mut().ctx.init(request_callback, arg);
        if let Err(err) = res {
            Err(err)
        } else {
            Ok(ctx)
        }
    }
}

extern fn request_callback(arg : AioCallbackArg) {
    unsafe {
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
