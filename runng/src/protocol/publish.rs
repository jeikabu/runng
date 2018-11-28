use aio::{NngAio, AioCallbackArg};
use futures::{
    sync::oneshot::{
        channel,
        Receiver,
        Sender,
    }
};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

pub struct Pub0 {
    socket: NngSocket
}

impl Pub0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_pub0_open(socket) };
        let socket_create_func = |socket| Pub0{ socket };
        nng_open(open_func, socket_create_func)
    }
}

#[derive(Debug,PartialEq)]
enum PublishState {
    Ready,
    Sending,
}

pub trait AsyncPublish {
    fn send(&mut self, msg: NngMsg) -> Receiver<NngReturn>;
}

pub struct AsyncPublishContext {
    aio: NngAio,
    state: PublishState,
    sender: Option<Sender<NngReturn>>
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

impl Socket for Pub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Pub0 {}
impl Listen for Pub0 {}
impl SendMsg for Pub0 {}

impl AsyncSocket for Pub0 {
    type ContextType = AsyncPublishContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket)
    }
}

extern fn publish_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPublishContext);
        
        trace!("callback Publish:{:?}", ctx.state);
        match ctx.state {
            PublishState::Ready => panic!(),
            PublishState::Sending => {
                let nng_aio = ctx.aio.nng_aio();
                let res = NngFail::from_i32(nng_aio_result(nng_aio));
                if let Err(_) = res {
                    // Nng requires that we retrieve the message and free it
                    let _ = NngMsg::new_msg(nng_aio_get_msg(nng_aio));
                }
                // Reset state before signaling completion
                ctx.state = PublishState::Ready;
                let res = ctx.sender.take().unwrap().send(res);
                if let Err(_) = res {
                    // Unable to send result.  Receiver probably went away.  Not necessarily a problem.
                }
            },
        }
    }
}


pub struct Push0 {
    socket: NngSocket
}

impl Push0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_push0_open(socket) }, 
            |socket| Push0{ socket }
        )
    }
}

impl Socket for Push0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Push0 {}
impl Listen for Push0 {}
impl SendMsg for Push0 {}

impl AsyncSocket for Push0 {
    type ContextType = AsyncPublishContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket)
    }
}


fn create_async_context(socket: NngSocket) -> NngResult<Box<AsyncPublishContext>> {
    let aio = NngAio::new(socket);
    let ctx = AsyncPublishContext {
        aio,
        state: PublishState::Ready,
        sender: None,
    };
    
    let mut ctx = Box::new(ctx);
    // This mess is needed to convert Box<_> to c_void
    let arg = ctx.as_mut() as *mut _ as AioCallbackArg;
    let res = ctx.as_mut().aio.init(publish_callback, arg);
    if let Err(err) = res {
        Err(err)
    } else {
        Ok(ctx)
    }
}