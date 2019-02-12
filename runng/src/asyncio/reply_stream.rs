//! Async request/reply

use crate::{
    aio::{Aio, AioCallbackArg, NngAio},
    asyncio::*,
    ctx::NngCtx,
    msg::NngMsg,
    *,
};
use futures::sync::{mpsc, oneshot};
use runng_sys::*;

#[derive(Debug, PartialEq)]
enum ReplyState {
    Receiving,
    Wait,
    Sending,
}

#[derive(Debug)]
struct ReplyContextAioArg {
    ctx: NngCtx,
    state: ReplyState,
    request_sender: mpsc::Sender<Result<NngMsg>>,
    reply_sender: Option<oneshot::Sender<Result<()>>>,
}

impl ReplyContextAioArg {
    pub fn create(
        socket: NngSocket,
        request_sender: mpsc::Sender<Result<NngMsg>>,
    ) -> Result<Box<Self>> {
        let ctx = NngCtx::create(socket)?;
        let arg = Self {
            ctx,
            state: ReplyState::Receiving,
            request_sender,
            reply_sender: None,
        };
        NngAio::register_aio(arg, reply_callback)
    }

    fn start_receive(&mut self) {
        if self.state != ReplyState::Receiving && self.state != ReplyState::Sending {
            panic!();
        }
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(self.ctx.ctx(), self.ctx.aio().nng_aio());
        }
    }

    pub fn reply(&mut self, msg: NngMsg, sender: oneshot::Sender<Result<()>>) {
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
#[derive(Debug)]
pub struct ReplyStreamHandle {
    aio_arg: Box<ReplyContextAioArg>,
    receiver: Option<mpsc::Receiver<Result<NngMsg>>>,
}

impl AsyncStreamContext for ReplyStreamHandle {
    fn create(socket: NngSocket, buffer: usize) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(buffer);
        let aio_arg = ReplyContextAioArg::create(socket, sender)?;
        let receiver = Some(receiver);
        let ctx = Self { aio_arg, receiver };
        Ok(ctx)
    }
}

/// Trait for asynchronous contexts that can receive a request and then send a reply.
pub trait AsyncReply {
    /// Asynchronously receive a request.
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>>;
    /// Asynchronously reply to previously received request.
    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>>;
}

impl AsyncReply for ReplyStreamHandle {
    fn receive(&mut self) -> Option<mpsc::Receiver<Result<NngMsg>>> {
        let receiver = self.receiver.take();
        if receiver.is_some() {
            self.aio_arg.start_receive();
        }
        receiver
    }

    fn reply(&mut self, msg: NngMsg) -> oneshot::Receiver<Result<()>> {
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
        ReplyState::Receiving => {
            let res = Error::from_i32(nng_aio_result(aio_nng));
            match res {
                Err(res) => {
                    match res {
                        Error::Errno(nng_errno_enum::NNG_ECLOSED)
                        | Error::Errno(nng_errno_enum::NNG_ECANCELED) => {
                            debug!("reply_callback {:?}", res);
                        }
                        _ => {
                            trace!("reply_callback::Err({:?})", res);
                            ctx.start_receive();
                        }
                    }

                    try_signal_complete(&mut ctx.request_sender, Err(res));
                }
                Ok(()) => {
                    let msg = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
                    // Reset state before signaling completion
                    ctx.state = ReplyState::Wait;
                    try_signal_complete(&mut ctx.request_sender, Ok(msg));
                }
            }
        }
        ReplyState::Wait => panic!(),
        ReplyState::Sending => {
            let res = Error::from_i32(nng_aio_result(aio_nng));
            if res.is_err() {
                // Nng requires we resume ownership of the message
                let _ = NngMsg::new_msg(nng_aio_get_msg(aio_nng));
            }

            let sender = ctx.reply_sender.take().unwrap();
            // Reset state and start receiving again before
            // signaling completion to avoid race condition where we say we're done, but
            // not yet ready for receive() to be called.
            ctx.start_receive();
            sender.send(res).unwrap();
        }
    }
}
