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
        open(|socket| unsafe { nng_rep0_open(socket) }, 
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
    fn reply(&mut self, NngMsg) -> NngResultFuture;
}

pub struct AsyncReplyContext {
    ctx: Option<NngCtx>,
    state: ReplyState,
    request_send: Option<oneshot::Sender<NngMsg>>,
    request_recv: Option<MsgFuture>,
    reply_send: Option<oneshot::Sender<NngReturn>>,
    reply_recv: Option<NngResultFuture>,
}

impl AsyncReplyContext {
    fn start_receive(&mut self) {
        let aionng = self.ctx.as_ref().unwrap().aio();
        let ctxnng = self.ctx.as_ref().unwrap().ctx();
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(ctxnng, aionng);
        }
    }
}

impl Context for AsyncReplyContext {
    fn new() -> Box<AsyncReplyContext> {
        let (request_send, request_recv) = oneshot::channel::<NngMsg>();
        let (reply_send, reply_recv) = oneshot::channel::<NngReturn>();
        let ctx = AsyncReplyContext {
            ctx: None,
            state: ReplyState::Receiving,
            request_send: Some(request_send), 
            request_recv: Some(request_recv),
            reply_send: Some(reply_send),
            reply_recv: Some(reply_recv),
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
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

    fn reply(&mut self, msg: NngMsg) -> NngResultFuture {
        if self.state != ReplyState::Wait {
            panic!();
        }
        
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();

            nng_aio_set_msg(aio, msg.take());
            self.state = ReplyState::Sending;
            nng_ctx_send(ctx, aio);
        }
        self.reply_recv.take().unwrap()
    }
}

impl Socket for Rep0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Listen for Rep0 {}
impl RecvMsg for Rep0 {}

pub trait AsyncReplySocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncReplyContext>>;
}


impl AsyncReplySocket for Rep0 {
    fn create_async_context(self) -> NngResult<Box<AsyncReplyContext>> {
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
                let res = nng_aio_result(aionng);
                let res = NngReturn::from_i32(res);
                //TODO: set error
                match res {
                    NngReturn::Fail(res) => {
                        match res {
                            NngFail::Err(NngError::ECLOSED) => {
                                println!("Closed");
                            },
                            NngFail::Err(_) => {
                                println!("Reply.Receive: {:?}", res);
                                ctx.start_receive();
                            },
                            NngFail::Unknown(res) => {
                                panic!(res);
                            },
                        }
                    },
                    NngReturn::Ok => {
                        let msg = nng_aio_get_msg(aionng);
                        let msg = NngMsg::new_msg(msg);
                        let sender = ctx.request_send.take().unwrap();
                        sender.send(msg).unwrap();
                        ctx.state = ReplyState::Wait;
                    }
                }
            },
            ReplyState::Wait => panic!(),
            ReplyState::Sending => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: destroy message and set error
                    panic!();
                }

                // No matter if sending reply succeeded/failed, start receiving again before
                // signaling completion to avoid race condition where we say we're done, but 
                // not yet ready for receive() to be called.
                ctx.start_receive();
                let sender = ctx.reply_send.take().unwrap();
                sender.send(NngReturn::from_i32(res)).unwrap();
            },
            
        }
    }
}
