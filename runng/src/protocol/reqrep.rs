use aio::{NngAio, AioCallback, AioCallbackArg};
use ctx::NngCtx;
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

pub struct Req0 {
    socket: NngSocket
}

pub struct Rep0 {
    socket: NngSocket
}


impl Req0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0{ socket };
        open(open_func, socket_create_func)
    }
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        open(|socket| unsafe { nng_rep0_open(socket) }, 
            |socket| Rep0{ socket }
        )
    }
}

#[derive(Debug,PartialEq)]
enum ReqRepState {
    Ready,
    Sending,
    Receiving,
}

#[derive(Debug,PartialEq)]
enum ReplyState {
    Receiving,
    Wait,
    Sending,
}

type MsgFuture = oneshot::Receiver<NngMsg>;
type NngResultFuture = oneshot::Receiver<NngReturn>;

pub trait AsyncReqRep {
    fn send(&mut self) -> MsgFuture;
}

pub trait AsyncReply {
    fn receive(&mut self) -> MsgFuture;
    fn reply(&mut self, NngMsg) -> NngResultFuture;
}

trait Context {
    fn new() -> Box<Self>;
    fn init(&mut self, Rc<NngAio>) -> NngResult<()>;
}

pub struct AsyncReqRepContext {
    ctx: Option<NngCtx>,
    state: ReqRepState,
    sender: Option<oneshot::Sender<NngMsg>>
}

impl Context for AsyncReqRepContext {
    fn new() -> Box<AsyncReqRepContext> {
        let ctx = AsyncReqRepContext {
            ctx: None,
            state: ReqRepState::Ready,
            sender: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
        let ctx = NngCtx::new(aio)?;
        self.ctx = Some(ctx);
        Ok(())
    }
}

impl AsyncReqRep for AsyncReqRepContext {
    fn send(&mut self) -> MsgFuture {
        if self.state != ReqRepState::Ready {
            panic!();
        }
        let (sender, receiver) = oneshot::channel::<NngMsg>();
        self.sender = Some(sender);
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();
            self.state = ReqRepState::Sending;

            let mut request: *mut nng_msg = std::ptr::null_mut();
            // TODO: check result != 0
            let res = nng_msg_alloc(&mut request, 0);
            nng_aio_set_msg(aio, request);

            nng_ctx_send(ctx, aio);
        }
        
        receiver
    }
}

pub struct AsyncReplyContext {
    ctx: Option<NngCtx>,
    state: ReplyState,
    requestSend: Option<oneshot::Sender<NngMsg>>,
    requestRecv: Option<MsgFuture>,
    replySend: Option<oneshot::Sender<NngReturn>>,
    replyRecv: Option<NngResultFuture>,
}

impl AsyncReplyContext {
    fn start_receive(&mut self) {
        let aionng = self.ctx.as_ref().unwrap().aio();
        let ctxnng = self.ctx.as_ref().unwrap().ctx();
        self.state = ReplyState::Receiving;
        unsafe {
            nng_ctx_recv(ctxnng, aionng);
        }
    }
}

impl Context for AsyncReplyContext {
    fn new() -> Box<AsyncReplyContext> {
        let (requestSend, requestRecv) = oneshot::channel::<NngMsg>();
        let (replySend, replyRecv) = oneshot::channel::<NngReturn>();
        let ctx = AsyncReplyContext {
            ctx: None,
            state: ReplyState::Receiving,
            requestSend: Some(requestSend), 
            requestRecv: Some(requestRecv),
            replySend: Some(replySend),
            replyRecv: Some(replyRecv),
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
        let ctx = NngCtx::new(aio)?;
        self.ctx = Some(ctx);
        self.start_receive();
        Ok(())
    }
}

impl AsyncReply for AsyncReplyContext {
    fn receive(&mut self) -> MsgFuture {
        if self.state != ReplyState::Receiving {
            panic!();
        }
        self.requestRecv.take().unwrap()
    }

    fn reply(&mut self, msg: NngMsg) -> NngResultFuture {
        if self.state != ReplyState::Wait {
            panic!();
        }
        
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();

            nng_aio_set_msg(aio, msg.take());
            self.state = ReplyState::Sending;
            nng_ctx_send(ctx, aio);
        }
        self.replyRecv.take().unwrap()
    }
}

impl Socket for Req0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}
impl Socket for Rep0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Dial for Req0 {}
impl SendMsg for Req0 {}
impl Listen for Rep0 {}
impl RecvMsg for Rep0 {}

fn create_async_context<T: Context>(socket: NngSocket, callback: AioCallback) -> NngResult<Box<T>> {
    let mut ctx = T::new();
    // This mess is needed to convert Box<_> to c_void
    let ctx_ptr = ctx.as_mut() as *mut _ as AioCallbackArg;
    let aio = NngAio::new(socket, callback, ctx_ptr)?;
    let aio = Rc::new(aio);
    (*ctx).init(aio.clone());
    Ok(ctx)
}

pub trait AsyncReqRepSocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>>;
}

impl AsyncReqRepSocket for Req0 {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>> {
        create_async_context(self.socket, request_callback)
    }
}

extern fn request_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncReqRepContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
        println!("callback Request:{:?}", ctx.state);
        match ctx.state {
            ReqRepState::Ready => panic!(),
            ReqRepState::Sending => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: destroy message and set error
                    ctx.state = ReqRepState::Ready;
                    panic!();
                    return;
                }

                ctx.state = ReqRepState::Receiving;
                nng_ctx_recv(ctxnng, aionng);
            },
            ReqRepState::Receiving => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: set error
                    ctx.state = ReqRepState::Ready;
                    panic!();
                    return;
                }
                let msg = nng_aio_get_msg(aionng);
                let msg = NngMsg::new_msg(msg);
                let sender = ctx.sender.take();
                ctx.state = ReqRepState::Ready;
                sender.unwrap().send(msg).unwrap();
            },
        }
    }
}

pub trait AsyncReplySocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncReplyContext>>;
}


impl AsyncReplySocket for Rep0 {
    fn create_async_context(self) -> NngResult<Box<AsyncReplyContext>> {
        create_async_context(self.socket, reply_callback)
    }
}

extern fn reply_callback(arg : AioCallbackArg) {
    unsafe {
        let ctx = &mut *(arg as *mut AsyncReplyContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
        println!("callback Reply:{:?}", ctx.state);
        match ctx.state {
            ReplyState::Receiving => {
                println!("1");
                let res = nng_aio_result(aionng);
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
                        let msg = nng_aio_get_msg(aionng);
                        let msg = NngMsg::new_msg(msg);
                        let sender = ctx.requestSend.take().unwrap();
                        sender.send(msg).unwrap();
                        ctx.state = ReplyState::Wait;
                    }
                }
            },
            ReplyState::Wait => panic!(),
            ReplyState::Sending => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: destroy message and set error
                    panic!();
                }

                // No matter if sending reply succeeded/failed, start receiving again before
                // signaling completion to avoid race condition where we say we're done, but 
                // not yet ready for receive() to be called.
                ctx.start_receive();
                let sender = ctx.replySend.take().unwrap();
                sender.send(NngReturn::from_i32(res)).unwrap();
            },
            
        }
    }
}
