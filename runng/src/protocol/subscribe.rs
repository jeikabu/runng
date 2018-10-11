use aio::{NngAio, AioCallback, AioCallbackArg};
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc, ffi::CString};
use super::*;

pub struct Sub0 {
    socket: NngSocket
}

impl Sub0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_sub0_open(socket) };
        let socket_create_func = |socket| Sub0{ socket };
        open(open_func, socket_create_func)
    }
}

#[derive(Debug,PartialEq)]
enum SubscribeState {
    Ready,
    Receiving,
}

pub trait AsyncSubscribe {
    fn receive(&mut self) -> Option<MsgFuture>;
}

pub struct AsyncSubscribeContext {
    aio: Option<Rc<NngAio>>,
    state: SubscribeState,
    promise: Option<MsgPromise>,
    future: Option<MsgFuture>,
}

impl AsyncSubscribeContext {
    fn start_receive(&mut self) {
        self.state = SubscribeState::Receiving;
        let (promise, future) = oneshot::channel::<MsgFutureType>();
        self.promise = Some(promise);
        self.future = Some(future);
        if let Some(ref mut aio) = self.aio {
            unsafe {
                nng_recv_aio(aio.socket(), aio.aio());
            }
        }
    }
    pub fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe {
            if let Some(ref aio) = self.aio {
                let opt = NNG_OPT_SUB_SUBSCRIBE.as_ptr() as *const ::std::os::raw::c_char;
                let topic_ptr = topic.as_ptr() as *const ::std::os::raw::c_void;
                let topic_size = std::mem::size_of_val(topic);
                let res = nng_setopt(aio.socket(), opt, topic_ptr, topic_size);
                NngReturn::from_i32(res)
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
            aio: None,
            state: SubscribeState::Ready,
            promise: None,
            future: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
        self.aio = Some(aio);
        self.start_receive();
        Ok(())
    }
}

impl AsyncSubscribe for AsyncSubscribeContext {
    fn receive(&mut self) -> Option<MsgFuture> {
        self.future.take()
    }
}

impl Socket for Sub0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Dial for Sub0 {}
impl RecvMsg for Sub0 {}

pub trait AsyncSubscribeSocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncSubscribeContext>>;
}

impl AsyncSubscribeSocket for Sub0 {
    fn create_async_context(self) -> NngResult<Box<AsyncSubscribeContext>> {
        create_async_context(self.socket, subscribe_callback)
    }
}

extern fn subscribe_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncSubscribeContext);
        
        println!("callback Subscribe:{:?}", ctx.state);
        match ctx.state {
            SubscribeState::Ready => panic!(),
            SubscribeState::Receiving => {
                let aio = ctx.aio.as_ref().map(|aio| aio.aio());
                if let Some(aio) = aio {
                    let res = NngReturn::from_i32(nng_aio_result(aio));
                    match res {
                        NngReturn::Fail(res) => {
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
                        NngReturn::Ok => {
                            let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
                            let promise = ctx.promise.take().unwrap();
                            promise.send(Ok(msg)).unwrap();
                            ctx.start_receive();
                        }
                    }
                } else {
                    panic!();
                }
            },
        }
    }
}
