use aio::{
    Aio,
    NngAio,
};
use runng_sys::*;
use super::*;

pub trait Ctx {
    fn ctx(&self) -> nng_ctx;
}

pub struct NngCtx {
    ctx: nng_ctx,
    aio: NngAio,
}

impl NngCtx {
    pub fn new(socket: NngSocket) -> NngResult<NngCtx> {
        let mut ctx = nng_ctx { id: 0 };
        let res = unsafe {
            nng_ctx_open(&mut ctx, socket.nng_socket())
        };
        NngFail::from_i32(res)?;
        let aio = NngAio::new(socket);
        let ctx = NngCtx {
            ctx,
            aio,
        };
        Ok(ctx)
    }

    pub fn init(&mut self, callback: AioCallback, arg: AioCallbackArg) -> NngReturn {
        self.aio.init(callback, arg)
    }
}

impl Ctx for NngCtx {
    fn ctx(&self) -> nng_ctx {
        self.ctx
    }
}

impl Aio for NngCtx {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

impl Drop for NngCtx {
    fn drop(&mut self) {
        unsafe {
            //debug!("NngCtx.drop {:x}", self.ctx as u64);
            nng_ctx_close(self.ctx);
        }
    }
}