use aio::{NngAio, AioCallback, AioCallbackArg};
use futures::{sync::oneshot};
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
    fn send(&mut self, msg: NngMsg) -> NngReturnFuture;
}

pub struct AsyncPublishContext {
    aio: Option<Rc<NngAio>>,
    state: PublishState,
    sender: Option<NngReturnPromise>
}

impl Context for AsyncPublishContext {
    fn new() -> Box<AsyncPublishContext> {
        let ctx = AsyncPublishContext {
            aio: None,
            state: PublishState::Ready,
            sender: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngReturn {
        self.aio = Some(aio);
        Ok(())
    }
}

impl AsyncPublish for AsyncPublishContext {
    fn send(&mut self, msg: NngMsg) -> NngReturnFuture {
        if self.state != PublishState::Ready {
            panic!();
        }
        let (sender, receiver) = oneshot::channel::<NngReturn>();
        self.sender = Some(sender);
        unsafe {
            if let Some(ref aio) = self.aio {
                self.state = PublishState::Sending;

                // Nng takes ownership of the message
                let msg = msg.take();
                nng_aio_set_msg(aio.aio(), msg);
                nng_send_aio(aio.nng_socket(), aio.aio());
            }
        }
        
        receiver
    }
}

impl Socket for Pub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}

impl Listen for Pub0 {}
impl SendMsg for Pub0 {}

impl AsyncSocket for Pub0 {
    type ContextType = AsyncPublishContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, publish_callback)
    }
}

extern fn publish_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPublishContext);
        
        println!("callback Publish:{:?}", ctx.state);
        match ctx.state {
            PublishState::Ready => panic!(),
            PublishState::Sending => {
                if let Some(ref mut aio) = ctx.aio {
                    let res = NngFail::from_i32(nng_aio_result(aio.aio()));
                    if let Err(_) = res {
                        // Nng requires that we retrieve the message and free it
                        let _ = NngMsg::new_msg(nng_aio_get_msg(aio.aio()));
                    }
                    ctx.state = PublishState::Ready;
                    ctx.sender.take().unwrap().send(res).unwrap();
                } else {
                    panic!();
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
}

impl Listen for Push0 {}
impl SendMsg for Push0 {}

impl AsyncSocket for Push0 {
    type ContextType = AsyncPublishContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, publish_callback)
    }
}
