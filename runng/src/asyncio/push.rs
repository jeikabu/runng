//! Async push and publish.

use super::*;
use log::debug;

#[derive(Debug, PartialEq)]
enum PushState {
    Ready,
    Sending,
}

#[derive(Debug)]
struct PushContextAioArg {
    aio: NngAio,
    state: PushState,
    sender: Option<oneshot::Sender<Result<()>>>,
    socket: NngSocket,
}

impl PushContextAioArg {
    pub fn new(socket: NngSocket) -> Result<AioArg<Self>> {
        NngAio::new(
            |aio| Self {
                aio,
                state: PushState::Ready,
                sender: None,
                socket,
            },
            publish_callback,
        )
    }

    pub fn send(&mut self, msg: NngMsg, sender: oneshot::Sender<Result<()>>) {
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
            nng_send_aio(self.socket.nng_socket(), nng_aio);
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

/// Async push context for push/pull pattern.
#[derive(Debug)]
pub struct PushAsyncHandle {
    aio_arg: AioArg<PushContextAioArg>,
}

impl AsyncContext for PushAsyncHandle {
    /// Create an asynchronous context using the specified socket.
    fn new(socket: NngSocket) -> Result<Self> {
        let aio_arg = PushContextAioArg::new(socket)?;
        Ok(Self { aio_arg })
    }
}

/// Trait for asynchronous contexts that can send a message.
pub trait AsyncPush {
    /// Asynchronously send a message.
    fn send(&mut self, msg: NngMsg) -> AsyncUnit;
}

impl AsyncPush for PushAsyncHandle {
    fn send(&mut self, msg: NngMsg) -> AsyncUnit {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.aio_arg.send(msg, sender);

        Box::pin(receiver.map(result::flatten_result))
    }
}

unsafe extern "C" fn publish_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut PushContextAioArg);

    trace!("callback Push:{:?}", ctx.state);
    match ctx.state {
        PushState::Ready => panic!(),
        PushState::Sending => {
            let nng_aio = ctx.aio.nng_aio();
            let res = nng_int_to_result(nng_aio_result(nng_aio));
            if let Err(ref err) = res {
                debug!("Push failed: {:?}", err);
                // Nng requires that we retrieve the message and free it
                let _ = NngMsg::from_raw(nng_aio_get_msg(nng_aio));
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
