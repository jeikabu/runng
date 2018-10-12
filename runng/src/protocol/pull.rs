use aio::{NngAio, AioCallback, AioCallbackArg};
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc, ffi::CString};
use super::*;

pub struct Pull0 {
    socket: NngSocket
}

impl Pull0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_pull0_open(socket) }, 
            |socket| Pull0{ socket }
        )
    }
}

#[derive(Debug,PartialEq)]
enum PullState {
    Ready,
    Receiving,
}

pub trait AsyncPull {
    fn receive(&mut self) -> Option<MsgFuture>;
}

pub struct AsyncPullContext {
    aio: Option<Rc<NngAio>>,
    state: PullState,
    promise: Option<MsgPromise>,
    future: Option<MsgFuture>,
}

impl AsyncPullContext {
    fn start_receive(&mut self) {
        self.state = PullState::Receiving;
        let (promise, future) = oneshot::channel::<MsgFutureType>();
        self.promise = Some(promise);
        self.future = Some(future);
        if let Some(ref mut aio) = self.aio {
            unsafe {
                nng_recv_aio(aio.nng_socket(), aio.aio());
            }
        }
    }
}

impl Context for AsyncPullContext {
    fn new() -> Box<Self> {
        let ctx = Self {
            aio: None,
            state: PullState::Ready,
            promise: None,
            future: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngReturn {
        self.aio = Some(aio);
        self.start_receive();
        Ok(())
    }
}

impl AsyncPull for AsyncPullContext {
    fn receive(&mut self) -> Option<MsgFuture> {
        self.future.take()
    }
}

impl Socket for Pull0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}

impl Dial for Pull0 {}
impl Listen for Pull0 {}
impl RecvMsg for Pull0 {}

impl AsyncSocket for Pull0 {
    type ContextType = AsyncPullContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, pull_callback)
    }
}

extern fn pull_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPullContext);
        
        println!("callback Subscribe:{:?}", ctx.state);
        match ctx.state {
            PullState::Ready => panic!(),
            PullState::Receiving => {
                let aio = ctx.aio.as_ref().map(|aio| aio.aio());
                if let Some(aio) = aio {
                    let res = NngFail::from_i32(nng_aio_result(aio));
                    match res {
                        Err(res) => {
                            match res {
                                NngFail::Err(NngError::ECLOSED) => {
                                    println!("Closed");
                                },
                                _ => {
                                    println!("Reply.Receive: {:?}", res);
                                    ctx.start_receive();
                                },
                            }
                            let promise = ctx.promise.take().unwrap();
                            promise.send(Err(res)).unwrap();
                        },
                        Ok(()) => {
                            let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
                            let promise = ctx.promise.take().unwrap();
                            // Make sure to reset state before signaling completion.  Otherwise
                            // have race-condition where receiver can receive None promise
                            ctx.start_receive();
                            promise.send(Ok(msg)).unwrap();
                        }
                    }
                } else {
                    panic!();
                }
            },
        }
    }
}



pub struct Sub0 {
    socket: NngSocket
}

impl Sub0 {
    pub fn open() -> NngResult<Self> {
        nng_open(
            |socket| unsafe { nng_sub0_open(socket) }, 
            |socket| Sub0{ socket }
        )
    }
}

pub struct AsyncSubscribeContext {
    ctx: AsyncPullContext
}

impl AsyncSubscribeContext {
    pub fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe {
            if let Some(ref aio) = self.ctx.aio {
                let opt = NNG_OPT_SUB_SUBSCRIBE.as_ptr() as *const ::std::os::raw::c_char;
                let topic_ptr = topic.as_ptr() as *const ::std::os::raw::c_void;
                let topic_size = std::mem::size_of_val(topic);
                let res = nng_setopt(aio.nng_socket(), opt, topic_ptr, topic_size);
                NngFail::from_i32(res)
            } else {
                panic!();
            }
        }
    }
    pub fn subscribe_str(&self, topic: &str) -> NngReturn {
        self.subscribe(topic.as_bytes())
    }
}

impl Context for AsyncSubscribeContext {
    fn new() -> Box<AsyncSubscribeContext> {
        let ctx = AsyncSubscribeContext {
            ctx: *AsyncPullContext::new()
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngReturn {
        self.ctx.init(aio)
    }
}

impl AsyncPull for AsyncSubscribeContext {
    fn receive(&mut self) -> Option<MsgFuture> {
        self.ctx.future.take()
    }
}

impl Socket for Sub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
}

impl Dial for Sub0 {}
impl RecvMsg for Sub0 {}

impl AsyncSocket for Sub0 {
    type ContextType = AsyncSubscribeContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        create_async_context(self.socket, pull_callback)
    }
}
