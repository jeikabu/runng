use aio::{NngAio, AioCallbackArg};
use ctx::NngCtx;
use futures::{
    sync::oneshot,
    sync::mpsc,
};
use msg::NngMsg;
use runng_sys::*;
use super::*;

#[derive(Debug,PartialEq)]
enum ReplyState {
    Receiving,
    Wait,
    Sending,
}

pub struct AsyncReplyContext {
    ctx: NngCtx,
    state: ReplyState,
    request_sender: Option<mpsc::Sender<NngResult<NngMsg>>>,
    reply_sender: Option<oneshot::Sender<NngReturn>>,
}

impl AsyncReplyContext {
    fn start_receive(&mut self) {
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(self.ctx.ctx(), self.ctx.aio().nng_aio());
        }
    }
}

impl AsyncContext for AsyncReplyContext {
    fn new(socket: NngSocket) -> Self {
        let ctx = NngCtx::new(socket).unwrap();
        Self {
            ctx,
            state: ReplyState::Receiving,
            request_sender: None,
            reply_sender: None,
        }
    }
    fn get_aio_callback() -> AioCallback {
        reply_callback
    }
}

impl Aio for AsyncReplyContext {
    fn aio(&self) -> &NngAio {
        self.ctx.aio()
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        self.ctx.aio_mut()
    }
}

pub trait AsyncReply {
    fn receive(&mut self) -> mpsc::Receiver<NngResult<NngMsg>>;
    fn reply(&mut self, NngMsg) -> oneshot::Receiver<NngReturn>;
}

impl AsyncReply for AsyncReplyContext {
    fn receive(&mut self) -> mpsc::Receiver<NngResult<NngMsg>> {
        if self.state != ReplyState::Receiving {
            panic!();
        }
        let (sender, receiver) = mpsc::channel(1024);
        self.request_sender = Some(sender);
        self.start_receive();
        receiver
    }

    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        if self.state != ReplyState::Wait {
            panic!();
        }
        
        let (sender, receiver) = oneshot::channel();
        self.reply_sender = Some(sender);
        unsafe {
            let aio = self.ctx.aio().nng_aio();

            self.state = ReplyState::Sending;
            // Nng assumes ownership of the message
            nng_aio_set_msg(aio, msg.take());
            nng_ctx_send(self.ctx.ctx(), aio);
        }
        receiver
    }
}

extern fn reply_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncReplyContext);
        let aio_nng = ctx.ctx.aio().nng_aio();
        trace!("callback Reply:{:?}", ctx.state);
        match ctx.state {
            ReplyState::Receiving => {
                let res = NngFail::from_i32(nng_aio_result(aio_nng));
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

                        try_signal_complete(&mut ctx.request_sender, Err(res));
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
                        // Reset state before signaling completion
                        ctx.state = ReplyState::Wait;
                        try_signal_complete(&mut ctx.request_sender, Ok(msg));
                    }
                }
            },
            ReplyState::Wait => panic!(),
            ReplyState::Sending => {
                let res = NngFail::from_i32(nng_aio_result(aio_nng));
                if let Err(_) = res {
                    // Nng requires we resume ownership of the message
                    let _ = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
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
