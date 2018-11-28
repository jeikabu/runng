use aio::{NngAio, AioCallbackArg};
use futures::{
    Sink,
    sync::mpsc::{
        channel,
        Receiver,
        Sender,
    }
    };
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
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
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>>;
}

pub struct AsyncPullContext {
    aio: NngAio,
    state: PullState,
    sender: Option<Sender<NngResult<NngMsg>>>,
}

impl AsyncPullContext {
    fn start_receive(&mut self) {
        self.state = PullState::Receiving;
        unsafe {
            nng_recv_aio(self.aio.nng_socket(), self.aio.nng_aio());
        }
    }
}

impl AsyncPull for AsyncPullContext {
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>> {
        let (sender, receiver) = channel::<NngResult<NngMsg>>(1024);
        self.sender = Some(sender);
        self.start_receive();
        receiver
    }
}

impl Socket for Pull0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Pull0 {}
impl Listen for Pull0 {}
impl RecvMsg for Pull0 {}

impl AsyncSocket for Pull0 {
    type ContextType = AsyncPullContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        let aio = NngAio::new(self.socket);
        let ctx = Self::ContextType {
            aio,
            state: PullState::Ready,
            sender: None,
        };
        
        let mut ctx = Box::new(ctx);
        // This mess is needed to convert Box<_> to c_void
        let arg = ctx.as_mut() as *mut _ as AioCallbackArg;
        let res = ctx.as_mut().aio.init(pull_callback, arg);
        if let Err(err) = res {
            Err(err)
        } else {
            Ok(ctx)
        }
    }
}

extern fn pull_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncPullContext);
        
        trace!("callback Subscribe:{:?}", ctx.state);
        match ctx.state {
            PullState::Ready => panic!(),
            PullState::Receiving => {
                let aio = ctx.aio.nng_aio();
                let res = NngFail::from_i32(nng_aio_result(aio));
                match res {
                    Err(res) => {
                        match res {
                            NngFail::Err(NngError::ECLOSED) => {
                                debug!("Closed");
                            },
                            _ => {
                                trace!("Reply.Receive: {:?}", res);
                                ctx.start_receive();
                            },
                        }
                        if let Some(ref mut sender) = ctx.sender {
                            let res = sender.try_send(Err(res));
                            if let Err(err) = res {
                                if err.is_disconnected() {
                                    sender.close();
                                } else {
                                    debug!("Send failed: {}", err);
                                }
                            }
                        }
                    },
                    Ok(()) => {
                        let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
                        // Make sure to reset state before signaling completion.  Otherwise
                        // have race-condition where receiver can receive None promise
                        ctx.start_receive();
                        if let Some(ref mut sender) = ctx.sender {
                            let res = sender.try_send(Ok(msg));
                            if let Err(err) = res {
                                if err.is_disconnected() {
                                    sender.close();
                                } else {
                                    debug!("Send failed: {}", err);
                                }
                            }
                        }
                    }
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

impl Subscribe for Sub0 {
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe {
            subscribe(self.socket.nng_socket(), topic)
        }
    }
}

pub trait Subscribe {
    fn subscribe(&self, topic: &[u8]) -> NngReturn;
    fn subscribe_str(&self, topic: &str) -> NngReturn {
        self.subscribe(topic.as_bytes())
    }
}

fn subscribe(socket: nng_socket, topic: &[u8]) -> NngReturn {
    unsafe {
        let opt = NNG_OPT_SUB_SUBSCRIBE.as_ptr() as *const ::std::os::raw::c_char;
        let topic_ptr = topic.as_ptr() as *const ::std::os::raw::c_void;
        let topic_size = std::mem::size_of_val(topic);
        let res = nng_setopt(socket, opt, topic_ptr, topic_size);
        NngFail::from_i32(res)
    }
}

pub struct AsyncSubscribeContext {
    ctx: AsyncPullContext,
}

impl Subscribe for AsyncSubscribeContext {
    fn subscribe(&self, topic: &[u8]) -> NngReturn {
        unsafe {
            subscribe(self.ctx.aio.nng_socket(), topic)
        }
    }
}

impl AsyncPull for AsyncSubscribeContext {
    fn receive(&mut self) -> Receiver<NngResult<NngMsg>> {
        let (sender, receiver) = channel::<NngResult<NngMsg>>(1024);
        self.ctx.sender = Some(sender);
        self.ctx.start_receive();
        receiver
    }
}

impl Socket for Sub0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn take(self) -> NngSocket {
        self.socket
    }
}

impl Dial for Sub0 {}
impl RecvMsg for Sub0 {}

impl AsyncSocket for Sub0 {
    type ContextType = AsyncSubscribeContext;
    fn create_async_context(self) -> NngResult<Box<Self::ContextType>> {
        let aio = NngAio::new(self.socket);
        let ctx = Self::ContextType {
            ctx: AsyncPullContext {
                aio,
                state: PullState::Ready,
                sender: None,
            }
        };
        
        let mut ctx = Box::new(ctx);
        // This mess is needed to convert Box<_> to c_void
        let arg = ctx.as_mut() as *mut _ as AioCallbackArg;
        let res = ctx.as_mut().ctx.aio.init(pull_callback, arg);
        if let Err(err) = res {
            Err(err)
        } else {
            Ok(ctx)
        }
    }
}
