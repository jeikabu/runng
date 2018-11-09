use aio::{NngAio, AioCallback, AioCallbackArg};
use ctx::NngCtx;
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

pub struct Rep0 {
    socket: NngSocket
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        nng_open(|socket| unsafe { nng_rep0_open(socket) }, 
            |socket| Rep0{ socket }
        )
    }
}

#[derive(Debug,PartialEq)]
enum ReplyState {
    Receiving,
    Wait,
    Sending,
}

pub trait AsyncReply {
    fn receive(&mut self) -> MsgFuture;
    fn reply(&mut self, NngMsg) -> NngReturnFuture;
}

pub struct AsyncReplyContext {
    ctx: Option<NngCtx>,
    state: ReplyState,
    request_send: Option<MsgPromise>,
    request_recv: Option<MsgFuture>,
    reply_send: Option<NngReturnPromise>,
    reply_recv: Option<NngReturnFuture>,
}

impl AsyncReplyContext {
    fn start_receive(&mut self) {
        let aionng = self.ctx.as_ref().unwrap().aio();
        let ctxnng = self.ctx.as_ref().unwrap().ctx();
        self.state = ReplyState::Receiving;
        let (request_send, request_recv) = oneshot::channel::<MsgFutureType>();
        let (reply_send, reply_recv) = oneshot::channel::<NngReturn>();
        self.request_send = Some(request_send);
        self.request_recv = Some(request_recv);
        self.reply_send = Some(reply_send);
        self.reply_recv = Some(reply_recv);
        unsafe {
            nng_ctx_recv(ctxnng, aionng);
        }
    }
}

impl Context for AsyncReplyContext {
    fn new() -> Box<AsyncReplyContext> {
        let ctx = AsyncReplyContext {
            ctx: None,
            state: ReplyState::Receiving,
            request_send: None, 
            request_recv: None,
            reply_send: None,
            reply_recv: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngReturn {
        let ctx = NngCtx::new(aio)?;
        self.ctx = Some(ctx);
        self.start_receive();
        Ok(())
    }
}

impl AsyncReply for AsyncReplyContext {
    fn receive(&mut self) -> MsgFuture {
        if self.state != ReplyState::Receiving {
            panic!();
        }
        self.request_recv.take().unwrap()
    }

    fn reply(&mut self, msg: NngMsg) -> NngReturnFuture {
        if self.state != ReplyState::Wait {
            panic!();
        }
        
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();

            self.state = ReplyState::Sending;
            // Nng assumes ownership of the message
            nng_aio_set_msg(aio, msg.take());
            nng_ctx_send(ctx, aio);
        }
        self.reply_recv.take().unwrap()
    }
}

impl Socket for Rep0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Listen for Rep0 {}
impl RecvMsg for Rep0 {}

impl AsyncSocket for Rep0 {
    type ContextType = AsyncReplyContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, reply_callback)
    }
}

extern fn reply_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncReplyContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
        println!("callback Reply:{:?}", ctx.state);
        match ctx.state {
            ReplyState::Receiving => {
                let res = NngFail::from_i32(nng_aio_result(aionng));
                match res {
                    Err(res) => {
                        match res {
                            NngFail::Err(NngError::ECLOSED) => {
                                println!("Closed");
                            },
                            _ => {
                                println!("Reply.Receive: {:?}", res);
                                ctx.start_receive();
                            },
                        }

                        let sender = ctx.request_send.take().unwrap();
                        sender.send(Err(res)).unwrap();
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aionng));
                        // Reset state before signaling completion
                        ctx.state = ReplyState::Wait;
                        let sender = ctx.request_send.take().unwrap();
                        sender.send(Ok(msg)).unwrap();
                    }
                }
            },
            ReplyState::Wait => panic!(),
            ReplyState::Sending => {
                let res = NngFail::from_i32(nng_aio_result(aionng));
                if let Err(_) = res {
                    // Nng requires we resume ownership of the message
                    let _ = NngMsg::new_msg(nng_aio_get_msg(aionng));
                }
                
                let sender = ctx.reply_send.take().unwrap();
                // Reset state and start receiving again before
                // signaling completion to avoid race condition where we say we're done, but 
                // not yet ready for receive() to be called.
                ctx.start_receive();
                sender.send(res).unwrap();
            },
            
        }
    }
}
