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
        open(open_func, socket_create_func)
    }
}

#[derive(Debug,PartialEq)]
enum PublishState {
    Ready,
    Sending,
}

pub trait AsyncPublish {
    fn send(&mut self) -> NngReturnFuture;
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
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
        self.aio = Some(aio);
        Ok(())
    }
}

impl AsyncPublish for AsyncPublishContext {
    fn send(&mut self) -> NngReturnFuture {
        if self.state != PublishState::Ready {
            panic!();
        }
        let (sender, receiver) = oneshot::channel::<NngReturn>();
        self.sender = Some(sender);
        unsafe {
            if let Some(ref aio) = self.aio {
                self.state = PublishState::Sending;

                let mut request: *mut nng_msg = std::ptr::null_mut();
                // TODO: check result != 0
                let res = nng_msg_alloc(&mut request, 0);
                nng_msg_append_u32(request, 0);
                nng_aio_set_msg(aio.aio(), request);
                nng_send_aio(aio.socket(), aio.aio());
            }
        }
        
        receiver
    }
}

impl Socket for Pub0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Listen for Pub0 {}
impl SendMsg for Pub0 {}

pub trait AsyncPublishSocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncPublishContext>>;
}

impl AsyncPublishSocket for Pub0 {
    fn create_async_context(self) -> NngResult<Box<AsyncPublishContext>> {
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
                    let res = NngReturn::from_i32(nng_aio_result(aio.aio()));
                    if let NngReturn::Fail(_) = res {
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
