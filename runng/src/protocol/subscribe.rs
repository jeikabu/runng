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
    fn receive(&mut self) -> MsgFuture;
}

pub struct AsyncSubscribeContext {
    aio: Option<Rc<NngAio>>,
    state: SubscribeState,
    sender: Option<oneshot::Sender<NngMsg>>
}

impl AsyncSubscribeContext {
    fn start_receive(&mut self) {
        self.state = SubscribeState::Receiving;
        if let Some(ref mut aio) = self.aio {
            unsafe {
                nng_recv_aio(aio.socket(), aio.aio());
            }
        }
    }
    pub fn subscribe(&self, name: &str) -> NngReturn {
        unsafe {
            if let Some(ref aio) = self.aio {
                let name = CString::new(name).unwrap();
                let name = name.as_bytes_with_nul().as_ptr() as *const i8;
                let topic: Vec<u32> = vec![0];
                let topic = topic.as_ptr() as *const ::std::os::raw::c_void;
                let res = nng_setopt(aio.socket(), name, topic, std::mem::size_of::<u32>());
                NngReturn::from_i32(res)
            } else {
                panic!();
            }
        }
    }
}

impl Context for AsyncSubscribeContext {
    fn new() -> Box<AsyncSubscribeContext> {
        let ctx = AsyncSubscribeContext {
            aio: None,
            state: SubscribeState::Ready,
            sender: None,
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
    fn receive(&mut self) -> MsgFuture {
        // if self.state != SubscribeState::Ready {
        //     panic!();
        // }
        let (sender, receiver) = oneshot::channel::<NngMsg>();
        self.sender = Some(sender);
        
        receiver
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
                    let res = nng_aio_result(aio);
                    let res = NngReturn::from_i32(res);
                    //TODO: set error
                    match res {
                        NngReturn::Fail(res) => {
                            match res {
                                NngFail::Err(NngError::ECLOSED) => {
                                    println!("Closed");
                                },
                                NngFail::Err(_) => {
                                    println!("Reply.Receive: {:?}", res);
                                    ctx.start_receive();
                                },
                                NngFail::Unknown(res) => {
                                    panic!(res);
                                },
                            }
                        },
                        NngReturn::Ok => {
                            let msg = nng_aio_get_msg(aio);
                            let msg = NngMsg::new_msg(msg);
                            ctx.sender.take().unwrap().send(msg).unwrap();
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
