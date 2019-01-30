//! Async request/reply

use super::*;
use crate::{
    aio::{Aio, AioCallbackArg, NngAio},
    ctx::NngCtx,
    msg::NngMsg,
    *,
};
use futures::sync::oneshot;
use runng_sys::*;
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
enum ReplyState {
    Idle,
    Receiving,
    Wait,
    Sending,
}

struct ReplyContextAioArg {
    ctx: NngCtx,
    state: ReplyState,
    queue: Mutex<WorkQueue>,
    reply_sender: Option<oneshot::Sender<NngReturn>>,
}

impl ReplyContextAioArg {
    pub fn create(socket: NngSocket) -> NngResult<Box<Self>> {
        let ctx = NngCtx::create(socket)?;
        let queue = Mutex::new(WorkQueue::default());
        let arg = Self {
            ctx,
            state: ReplyState::Idle,
            queue,
            reply_sender: None,
        };
        let mut context = NngAio::register_aio(arg, reply_callback);
        if let Ok(ref mut context) = context {
            context.receive();
        }
        context
    }

    fn receive(&mut self) {
        if self.state != ReplyState::Idle {
            panic!();
        }
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(self.ctx.ctx(), self.ctx.aio().nng_aio());
        }
    }

    pub fn reply(&mut self, msg: NngMsg, sender: oneshot::Sender<NngReturn>) {
        if self.state != ReplyState::Wait {
            panic!();
        }

        self.reply_sender = Some(sender);
        unsafe {
            let aio = self.ctx.aio().nng_aio();

            self.state = ReplyState::Sending;
            // Nng assumes ownership of the message
            nng_aio_set_msg(aio, msg.take());
            nng_ctx_send(self.ctx.ctx(), aio);
        }
    }
}

impl Aio for ReplyContextAioArg {
    fn aio(&self) -> &NngAio {
        self.ctx.aio()
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        self.ctx.aio_mut()
    }
}

/// Asynchronous context for reply socket.
pub struct ReplyAsyncHandle {
    aio_arg: Box<ReplyContextAioArg>,
}

impl AsyncContext for ReplyAsyncHandle {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let aio_arg = ReplyContextAioArg::create(socket)?;
        Ok(Self { aio_arg })
    }
}

/// Trait for asynchronous contexts that can receive a request and then send a reply.
pub trait ReplyAsync {
    // FIXME: Can change this to -> impl Future later?
    /// Asynchronously receive a request.
    fn receive(&mut self) -> Box<dyn Future<Item = NngResult<NngMsg>, Error = oneshot::Canceled>>;
    /// Asynchronously reply to previously received request.
    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn>;
}

impl ReplyAsync for ReplyAsyncHandle {
    fn receive(&mut self) -> Box<dyn Future<Item = NngResult<NngMsg>, Error = oneshot::Canceled>> {
        let mut queue = self.aio_arg.queue.lock().unwrap();
        if let Some(item) = queue.ready.pop_front() {
            Box::new(future::ok(item))
        } else {
            let (sender, receiver) = oneshot::channel();
            queue.waiting.push_back(sender);
            Box::new(receiver)
        }
    }

    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        let (sender, receiver) = oneshot::channel();
        self.aio_arg.reply(msg, sender);
        receiver
    }
}

unsafe extern "C" fn reply_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut ReplyContextAioArg);
    let aio_nng = ctx.ctx.aio().nng_aio();
    trace!("reply_callback::{:?}", ctx.state);
    match ctx.state {
        ReplyState::Idle => panic!(),
        ReplyState::Receiving => {
            let res = NngFail::from_i32(nng_aio_result(aio_nng));
            match res {
                Err(res) => {
                    match res {
                        NngFail::Err(nng_errno_enum::NNG_ECLOSED)
                        | NngFail::Err(nng_errno_enum::NNG_ECANCELED) => {
                            debug!("reply_callback {:?}", res);
                        }
                        _ => {
                            trace!("reply_callback::Err({:?})", res);
                            ctx.receive();
                        }
                    }

                    ctx.queue.lock().unwrap().push_back(Err(res));
                }
                Ok(()) => {
                    let msg = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
                    // Reset state before signaling completion
                    ctx.state = ReplyState::Wait;
                    ctx.queue.lock().unwrap().push_back(Ok(msg));
                }
            }
        }
        ReplyState::Wait => panic!(),
        ReplyState::Sending => {
            let res = NngFail::from_i32(nng_aio_result(aio_nng));
            if res.is_err() {
                // Nng requires we resume ownership of the message
                let _ = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
            }

            let sender = ctx.reply_sender.take().unwrap();
            // Reset state and start receiving again before
            // signaling completion to avoid race condition where we say we're done, but
            // not yet ready for receive() to be called.
            ctx.state = ReplyState::Idle;
            ctx.receive();
            sender.send(res).unwrap();
        }
    }
}
