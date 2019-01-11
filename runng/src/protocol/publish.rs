//! Async publish/subscribe

use crate::{
    aio::{AioCallbackArg, NngAio},
    msg::NngMsg,
    protocol::AsyncContext,
    *,
};
use futures::sync::oneshot::{channel, Receiver, Sender};
use log::debug;
use runng_sys::*;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
enum PublishState {
    Ready,
    Sending,
}

struct PublishContextAioArg {
    aio: NngAio,
    state: PublishState,
    sender: Option<Sender<NngReturn>>,
}

impl PublishContextAioArg {
    pub fn create(socket: Arc<NngSocket>) -> NngResult<Box<Self>> {
        let aio = NngAio::new(socket);
        let arg = Self {
            aio,
            state: PublishState::Ready,
            sender: None,
        };
        NngAio::register_aio(arg, publish_callback)
    }

    pub fn send(&mut self, msg: NngMsg, sender: Sender<NngReturn>) {
        if self.state != PublishState::Ready {
            panic!();
        }
        self.sender = Some(sender);
        unsafe {
            self.state = PublishState::Sending;

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

impl Aio for PublishContextAioArg {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

/// Asynchronous context for publish socket.
pub struct AsyncPublishContext {
    aio_arg: Box<PublishContextAioArg>,
}

impl AsyncContext for AsyncPublishContext {
    /// Create an asynchronous context using the specified socket.
    fn create(socket: Arc<NngSocket>) -> NngResult<Self> {
        let aio_arg = PublishContextAioArg::create(socket)?;
        Ok(Self { aio_arg })
    }
}

/// Trait for asynchronous contexts that can send a message.
pub trait AsyncPublish {
    /// Asynchronously send a message.
    fn send(&mut self, msg: NngMsg) -> Receiver<NngReturn>;
}

impl AsyncPublish for AsyncPublishContext {
    fn send(&mut self, msg: NngMsg) -> Receiver<NngReturn> {
        let (sender, receiver) = channel::<NngReturn>();
        self.aio_arg.send(msg, sender);

        receiver
    }
}

unsafe extern "C" fn publish_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut PublishContextAioArg);

    trace!("callback Publish:{:?}", ctx.state);
    match ctx.state {
        PublishState::Ready => panic!(),
        PublishState::Sending => {
            let nng_aio = ctx.aio.nng_aio();
            let res = NngFail::from_i32(nng_aio_result(nng_aio));
            if let Err(ref err) = res {
                debug!("Publish failed: {:?}", err);
                // Nng requires that we retrieve the message and free it
                let _ = NngMsg::new_msg(nng_aio_get_msg(nng_aio));
            }
            // Reset state before signaling completion
            ctx.state = PublishState::Ready;
            let res = ctx.sender.take().unwrap().send(res);
            if let Err(ref err) = res {
                // Unable to send result.  Receiver probably went away.  Not necessarily a problem.
                debug!("Send finish failed: {:?}", err);
            }
        }
    }
}
