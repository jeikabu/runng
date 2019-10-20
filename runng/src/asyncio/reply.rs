//! Async request/reply

use super::*;
use crate::ctx::NngCtx;
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
enum ReplyState {
    Idle,
    Receiving,
    Wait,
    Sending,
}

#[derive(Debug)]
struct ReplyContextAioArg {
    aio: NngAio,
    ctx: NngCtx,
    queue: Mutex<WorkQueue>,
    reply_sender: Option<oneshot::Sender<Result<()>>>,
    socket: NngSocket,
    state: ReplyState,
}

impl ReplyContextAioArg {
    pub fn new(socket: NngSocket) -> Result<AioArg<Self>> {
        let ctx = NngCtx::new(socket.clone())?;
        let queue = Mutex::new(WorkQueue::default());
        let mut context = NngAio::new(
            |aio| Self {
                aio,
                ctx,
                queue,
                reply_sender: None,
                socket,
                state: ReplyState::Idle,
            },
            reply_callback,
        )?;

        context.receive();
        Ok(context)
    }

    fn receive(&mut self) {
        if self.state != ReplyState::Idle {
            panic!();
        }
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(self.ctx.ctx(), self.aio.nng_aio());
        }
    }

    pub fn reply(&mut self, msg: NngMsg, sender: oneshot::Sender<Result<()>>) {
        if self.state != ReplyState::Wait {
            panic!();
        }

        self.reply_sender = Some(sender);
        unsafe {
            let aio = self.aio.nng_aio();

            self.state = ReplyState::Sending;
            // Nng assumes ownership of the message
            nng_aio_set_msg(aio, msg.take());
            nng_ctx_send(self.ctx.ctx(), aio);
        }
    }
}

impl Aio for ReplyContextAioArg {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Async reply context for request/reply pattern.
#[derive(Debug)]
pub struct ReplyAsyncHandle {
    aio_arg: AioArg<ReplyContextAioArg>,
}

impl AsyncContext for ReplyAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let aio_arg = ReplyContextAioArg::new(socket)?;
        Ok(Self { aio_arg })
    }
}

/// Trait for asynchronous contexts that can receive a request and then send a reply.
pub trait ReplyAsync {
    // FIXME: Can change this to -> impl Future later?
    /// Asynchronously receive a request.
    fn receive(&mut self) -> AsyncMsg;
    /// Asynchronously reply to previously received request.
    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>>;
}

impl ReplyAsync for ReplyAsyncHandle {
    fn receive(&mut self) -> AsyncMsg {
        let mut queue = self.aio_arg.queue.lock().unwrap();
        if let Some(item) = queue.ready.pop_front() {
            Box::pin(future::ready(item))
        } else {
            let (sender, receiver) = oneshot::channel();
            queue.waiting.push_back(sender);
            let receiver = receiver.map(result::flatten_result);
            Box::pin(receiver)
        }
    }

    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>> {
        let (sender, receiver) = oneshot::channel();
        self.aio_arg.reply(msg, sender);
        receiver
    }
}

unsafe extern "C" fn reply_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut ReplyContextAioArg);
    let aio_nng = ctx.aio.nng_aio();
    trace!("reply_callback::{:?}", ctx.state);
    match ctx.state {
        ReplyState::Idle => panic!(),
        ReplyState::Receiving => {
            let res = nng_int_to_result(nng_aio_result(aio_nng));
            match res {
                Err(res) => {
                    match res {
                        Error::Errno(NngErrno::ECLOSED) | Error::Errno(NngErrno::ECANCELED) => {
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
                    let msg = NngMsg::from_raw(nng_aio_get_msg(aio_nng));
                    // Reset state before signaling completion
                    ctx.state = ReplyState::Wait;
                    ctx.queue.lock().unwrap().push_back(Ok(msg));
                }
            }
        }
        ReplyState::Wait => panic!(),
        ReplyState::Sending => {
            let res = nng_int_to_result(nng_aio_result(aio_nng));
            if res.is_err() {
                // Nng requires we resume ownership of the message
                let _ = NngMsg::from_raw(nng_aio_get_msg(aio_nng));
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
