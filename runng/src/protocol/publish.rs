//! Async publish/subscribe

use crate::{
    aio::{AioCallbackArg, NngAio},
    msg::NngMsg,
    protocol::AsyncContext,
    *,
};
use futures::sync::oneshot::{channel, Receiver, Sender};
use runng_sys::*;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
enum PublishState {
    Ready,
    Sending,
}

/// Asynchronous context for publish socket.
pub struct AsyncPublishContext {
    aio: NngAio,
    state: PublishState,
    sender: Option<Sender<NngReturn>>,
}

impl AsyncContext for AsyncPublishContext {
    /// Create an asynchronous context using the specified socket.
    fn new(socket: Arc<NngSocket>) -> NngResult<Self> {
        let aio = NngAio::new(socket);
        let ctx = Self {
            aio,
            state: PublishState::Ready,
            sender: None,
        };
        Ok(ctx)
    }
    fn get_aio_callback() -> AioCallback {
        publish_callback
    }
}

impl Aio for AsyncPublishContext {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Trait for asynchronous contexts that can send a message.
pub trait AsyncPublish {
    /// Asynchronously send a message.
    fn send(&mut self, msg: NngMsg) -> Receiver<NngReturn>;
}

impl AsyncPublish for AsyncPublishContext {
    fn send(&mut self, msg: NngMsg) -> Receiver<NngReturn> {
        if self.state != PublishState::Ready {
            panic!();
        }
        let (sender, receiver) = channel::<NngReturn>();
        self.sender = Some(sender);
        unsafe {
            self.state = PublishState::Sending;

            // Nng takes ownership of the message
            let msg = msg.take();
            let nng_aio = self.aio.nng_aio();
            nng_aio_set_msg(nng_aio, msg);
            nng_send_aio(self.aio.nng_socket(), nng_aio);
        }

        receiver
    }
}

extern "C" fn publish_callback(arg: AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPublishContext);

        trace!("callback Publish:{:?}", ctx.state);
        match ctx.state {
            PublishState::Ready => panic!(),
            PublishState::Sending => {
                let nng_aio = ctx.aio.nng_aio();
                let res = NngFail::from_i32(nng_aio_result(nng_aio));
                if res.is_err() {
                    // Nng requires that we retrieve the message and free it
                    let _ = NngMsg::new_msg(nng_aio_get_msg(nng_aio));
                }
                // Reset state before signaling completion
                ctx.state = PublishState::Ready;
                let res = ctx.sender.take().unwrap().send(res);
                if res.is_err() {
                    // Unable to send result.  Receiver probably went away.  Not necessarily a problem.
                }
            }
        }
    }
}
