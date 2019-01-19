//! Protocol contexts

use crate::{
    aio::{Aio, NngAio},
    *,
};
use log::trace;
use runng_sys::*;

/// Type which exposes a `NngCtx`.
pub trait Ctx {
    /// Obtain under-lying `NngCtx`.
    fn ctx(&self) -> nng_ctx;
}

/// Wraps `nng_ctx` and its associated `NngAio`.  See [nng_ctx](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx.5).
pub struct NngCtx {
    ctx: nng_ctx,
    aio: NngAio,
}

impl NngCtx {
    /// Creates a new context using the specified socket.  See [nng_ctx_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx_open.3).
    pub fn create(socket: NngSocket) -> NngResult<Self> {
        let mut ctx = nng_ctx { id: 0 };
        let res = unsafe { nng_ctx_open(&mut ctx, socket.nng_socket()) };
        NngFail::from_i32(res)?;
        let aio = NngAio::new(socket);
        let ctx = NngCtx { ctx, aio };
        Ok(ctx)
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
            if self.ctx.id != 0 {
                trace!("NngCtx.drop {:x}", self.ctx.id);
                nng_ctx_close(self.ctx);
            }
        }
    }
}
