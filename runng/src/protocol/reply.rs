use aio::{NngAio, AioCallbackArg};
use ctx::NngCtx;
use futures::{
    Sink,
    sync::oneshot,
    sync::mpsc,
    };
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
    fn receive(&mut self) -> mpsc::Receiver<NngResult<NngMsg>>;
    fn reply(&mut self, NngMsg) -> oneshot::Receiver<NngReturn>;
}

pub struct AsyncReplyContext {
    ctx: Option<NngCtx>,
    state: ReplyState,
    request_sender: Option<mpsc::Sender<NngResult<NngMsg>>>,
    reply_sender: Option<oneshot::Sender<NngReturn>>,
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
        let ctx = AsyncReplyContext {
            ctx: None,
            state: ReplyState::Receiving,
            request_sender: None,
            reply_sender: None,
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
    fn receive(&mut self) -> mpsc::Receiver<NngResult<NngMsg>> {
        if self.state != ReplyState::Receiving {
            panic!();
        }
        let (sender, receiver) = mpsc::channel(1024);
        self.request_sender = Some(sender);
        receiver
    }

    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        if self.state != ReplyState::Wait {
            panic!();
        }
        
        let (sender, receiver) = oneshot::channel();
        self.reply_sender = Some(sender);
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();

            self.state = ReplyState::Sending;
            // Nng assumes ownership of the message
            nng_aio_set_msg(aio, msg.take());
            nng_ctx_send(ctx, aio);
        }
        receiver
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
        trace!("callback Reply:{:?}", ctx.state);
        match ctx.state {
            ReplyState::Receiving => {
                let res = NngFail::from_i32(nng_aio_result(aionng));
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

                        if let Some(ref mut sender) = ctx.request_sender {
                            let res = sender.try_send(Err(res));
                            if let Err(err) = res {
                                if err.is_disconnected() {
                                    sender.close();
                                } else {
                                    debug!("Send failed: {}", err);
                                }
                            }
                        }
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aionng));
                        // Reset state before signaling completion
                        ctx.state = ReplyState::Wait;
                        if let Some(ref mut sender) = ctx.request_sender {
                            let res = sender.try_send(Ok(msg));
                            if let Err(err) = res {
                                if err.is_disconnected() {
                                    // Not an error?
                                } else {
                                    debug!("Receive failed: {}", err);
                                }
                            }
                        }
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
                
                let sender = ctx.reply_sender.take().unwrap();
                // Reset state and start receiving again before
                // signaling completion to avoid race condition where we say we're done, but 
                // not yet ready for receive() to be called.
                ctx.start_receive();
                sender.send(res).unwrap();
            },
            
        }
    }
}
