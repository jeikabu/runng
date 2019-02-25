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
#[derive(Debug)]
pub struct NngCtx {
    ctx: nng_ctx,
    aio: NngAio,
}

impl NngCtx {
    /// Creates a new context using the specified socket.  See [nng_ctx_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx_open.3).
    pub fn create(socket: NngSocket) -> Result<Self> {
        let mut ctx = nng_ctx::default();
        let res = unsafe { nng_ctx_open(&mut ctx, socket.nng_socket()) };
        nng_int_to_result(res)?;
        let aio = NngAio::new(socket);
        let ctx = NngCtx { ctx, aio };
        Ok(ctx)
    }

    /// See [nng_ctx_id](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_ctx_id(self.ctx) }
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
            let id = self.id();
            if id > 0 {
                trace!("NngCtx.drop {:x}", id);
                nng_ctx_close(self.ctx);
            }
        }
    }
}
