use ctx::NngCtx;
use aio::NngAio;
use runng_sys::*;
use super::*;

use std::{
    rc::Rc,
};

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
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();
            self.state = ReqRepState::Sending;

            let mut request: *mut nng_msg = std::ptr::null_mut();
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
        self.socket.socket()
    }
}
impl Socket for Rep0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
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
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
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