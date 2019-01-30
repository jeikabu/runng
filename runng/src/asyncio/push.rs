//! Async publish/subscribe

use crate::{
    aio::{AioCallbackArg, NngAio},
    asyncio::*,
    msg::NngMsg,
    *,
};
use futures::sync::oneshot;
use log::debug;
use runng_sys::*;

#[derive(Debug, PartialEq)]
enum PushState {
    Ready,
    Sending,
}

struct PushContextAioArg {
    aio: NngAio,
    state: PushState,
    sender: Option<oneshot::Sender<NngReturn>>,
}

impl PushContextAioArg {
    pub fn create(socket: NngSocket) -> NngResult<Box<Self>> {
        let aio = NngAio::new(socket);
        let arg = Self {
            aio,
            state: PushState::Ready,
            sender: None,
        };
        NngAio::register_aio(arg, publish_callback)
    }

    pub fn send(&mut self, msg: NngMsg, sender: oneshot::Sender<NngReturn>) {
        if self.state != PushState::Ready {
            panic!();
        }
        self.sender = Some(sender);
        unsafe {
            self.state = PushState::Sending;

            // Nng takes ownership of the message
            let msg = msg.take();
            if msg.is_null() {
                panic!();
            }
            let nng_aio = self.aio.nng_aio();
            nng_aio_set_msg(nng_aio, msg);
            nng_send_aio(self.aio.nng_socket(), nng_aio);
        }
    }
}

impl Aio for PushContextAioArg {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Asynchronous context for publish socket.
pub struct PushAsyncHandle {
    aio_arg: Box<PushContextAioArg>,
}

impl AsyncContext for PushAsyncHandle {
    /// Create an asynchronous context using the specified socket.
    fn create(socket: NngSocket) -> NngResult<Self> {
        let aio_arg = PushContextAioArg::create(socket)?;
        Ok(Self { aio_arg })
    }
}

/// Trait for asynchronous contexts that can send a message.
pub trait AsyncPush {
    /// Asynchronously send a message.
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn>;
}

impl AsyncPush for PushAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> oneshot::Receiver<NngReturn> {
        let (sender, receiver) = oneshot::channel::<NngReturn>();
        self.aio_arg.send(msg, sender);

        receiver
    }
}

unsafe extern "C" fn publish_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut PushContextAioArg);

    trace!("callback Push:{:?}", ctx.state);
    match ctx.state {
        PushState::Ready => panic!(),
        PushState::Sending => {
            let nng_aio = ctx.aio.nng_aio();
            let res = NngFail::from_i32(nng_aio_result(nng_aio));
            if let Err(ref err) = res {
                debug!("Push failed: {:?}", err);
                // Nng requires that we retrieve the message and free it
                let _ = NngMsg::new_msg(nng_aio_get_msg(nng_aio));
            }
            // Reset state before signaling completion
            ctx.state = PushState::Ready;
            let res = ctx.sender.take().unwrap().send(res);
            if let Err(ref err) = res {
                // Unable to send result.  Receiver probably went away.  Not necessarily a problem.
                debug!("Send finish failed: {:?}", err);
            }
        }
    }
}
