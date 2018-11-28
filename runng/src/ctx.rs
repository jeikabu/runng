use aio::{
    Aio,
    NngAio,
};
use runng_sys::*;
use super::*;

use std::{
    rc::Rc,
};

pub trait Ctx {
    fn ctx(&self) -> nng_ctx;
}

pub struct NngCtx {
    ctx: nng_ctx,
    aio: Rc<NngAio>
}

impl NngCtx {
    pub fn new(aio: Rc<NngAio>) -> NngResult<NngCtx> {
        let mut ctx = nng_ctx { id: 0 };
        let res = unsafe {
            nng_ctx_open(&mut ctx, aio.nng_socket())
        };
        NngFail::succeed_then(res, || NngCtx { ctx, aio })
    }
}

impl Ctx for NngCtx {
    fn ctx(&self) -> nng_ctx {
        self.ctx
    }
}

impl Aio for NngCtx {
    fn aio(&self) -> *mut nng_aio {
        self.aio.aio()
    }
}

impl Drop for NngCtx {
    fn drop(&mut self) {
        unsafe {
            debug!("NngCtx.drop {:x}", self.ctx as u64);
            nng_ctx_close(self.ctx);
        }
    }
}