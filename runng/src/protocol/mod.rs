use runng_sys::*;
use super::*;
use std::{
    cmp::PartialEq,
    rc::Rc,
};


pub struct Req0 {
    socket: NngSocket
}

pub struct Rep0 {
    socket: NngSocket
}

impl Req0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = NngSocket::new();
        let res = unsafe { nng_req0_open(&mut socket.socket) };
        if res == 0 {
            Ok(Req0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        let mut socket = NngSocket::new();
        let res = unsafe { nng_rep0_open(&mut socket.socket) };
        if res == 0 {
            Ok(Rep0 { socket } )
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}

struct NngAio {
    aio: *mut nng_aio,
    socket: NngSocket,
}

type AioCallback = unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void);

impl NngAio {
    fn new(socket: NngSocket, callback: AioCallback, arg: *mut ::std::os::raw::c_void) -> NngResult<NngAio> {
        unsafe {
            let mut tmp_aio = nng_aio::new();
            let mut tmp_aio = &mut tmp_aio as *mut nng_aio;
            //https://doc.rust-lang.org/stable/book/first-edition/ffi.html#callbacks-from-c-code-to-rust-functions
            let res = nng_aio_alloc(&mut tmp_aio, Some(callback), arg);
            if res != 0 {
                Err(NngFail::from_i32(res))
            } else {
                let aio = NngAio {
                    aio: tmp_aio,
                    socket
                };
                NngReturn::from(res, aio)
            }
        }
    }
}

impl Drop for NngAio {
    fn drop(&mut self) {
        unsafe {
            nng_aio_free(self.aio);
        }
    }
}

pub struct NngCtx {
    ctx: nng_ctx,
    aio: Rc<NngAio>
}

impl NngCtx {
    fn new(aio: Rc<NngAio>) -> NngResult<NngCtx> {
        let mut ctx = nng_ctx { id: 0 };
        let res = unsafe {
            nng_ctx_open(&mut ctx, aio.socket.socket)
        };
        if res == 0 {
            Ok(NngCtx { ctx, aio })
        } else {
            Err(NngFail::from_i32(res))
        }
    }
}

#[derive(PartialEq)]
enum ReqRepState {
    Ready,
    Sending,
    Receiving,
}

pub trait AsyncReqRep {
    fn send(&mut self);
}
pub struct AsyncReqRepContext {
    ctx: Option<NngCtx>,
    state: ReqRepState,
}
impl AsyncReqRepContext {
    fn new() -> Box<AsyncReqRepContext> {
        let ctx = AsyncReqRepContext {
            ctx: None,
            state: ReqRepState::Ready,
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
    fn send(&mut self) {
        if self.state != ReqRepState::Ready {
            panic!();
        }
        
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio.aio;
            let ctx = self.ctx.as_ref().unwrap().ctx;
            self.state = ReqRepState::Sending;

            let mut request = nng_msg::new();
            let mut request = &mut request as *mut nng_msg;
            // TODO: check result != 0
            let res = nng_msg_alloc(&mut request, 0);
            nng_aio_set_msg(aio, request);

            nng_ctx_send(ctx, aio);
        }
    }
}

pub trait AsyncReqRepSocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>>;
}

impl Socket for Req0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket
    }
}
impl Socket for Rep0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket
    }
}

impl Dial for Req0 {}
impl Send for Req0 {}
impl Listen for Rep0 {}
impl Recv for Rep0 {}

extern fn callback(arg : *mut ::std::os::raw::c_void) {
    unsafe {
        println!("callback {:?}", arg);
        let ctx = &mut *(arg as *mut AsyncReqRepContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio.aio;
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx;
        match ctx.state {
            ReqRepState::Ready => panic!(),
            ReqRepState::Sending => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: destroy message and set error
                    ctx.state = ReqRepState::Ready;
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
                    return;
                }
                let msg = nng_aio_get_msg(aionng);
                //TODO: future returns message
                ctx.state = ReqRepState::Ready;
            },
        }
    }
    
}

impl AsyncReqRepSocket for Req0 {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>> {
        let mut ctx = AsyncReqRepContext::new();
        // This mess is needed to convert Box<_> to c_void
        let ctx_ptr = ctx.as_mut() as *mut _ as *mut std::os::raw::c_void;
        let aio = NngAio::new(self.socket, callback, ctx_ptr)?;
        let aio = Rc::new(aio);
        ctx.init(aio.clone());
        Ok(ctx)
    }
}