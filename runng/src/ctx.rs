//! Protocol contexts (`nng_ctx`).

use crate::*;
use log::trace;
use runng_sys::*;

/// Type which exposes a [`NngCtx`](struct.NngCtx.html).
pub trait Ctx {
    /// Obtain under-lying `NngCtx`.
    fn ctx(&self) -> nng_ctx;
}

/// Handle to `nng_ctx`.  See [nng_ctx](https://nng.nanomsg.org/man/v1.2.2/nng_ctx.5).
#[derive(Debug)]
pub struct NngCtx {
    ctx: nng_ctx,
    // FIXME: should ctx keep a reference to the socket?
}

impl NngCtx {
    /// Creates a new context using the specified socket.  See [nng_ctx_open](https://nng.nanomsg.org/man/v1.2.2/nng_ctx_open.3).
    pub fn new(socket: NngSocket) -> Result<Self> {
        let mut ctx = nng_ctx::default();
        let res = unsafe { nng_ctx_open(&mut ctx, socket.nng_socket()) };
        nng_int_to_result(res)?;
        let ctx = Self { ctx };
        Ok(ctx)
    }

    /// See [nng_ctx_id](https://nng.nanomsg.org/man/v1.2.2/nng_ctx_id.3).
    pub fn id(&self) -> i32 {
        unsafe { nng_ctx_id(self.ctx) }
    }
}

impl Ctx for NngCtx {
    fn ctx(&self) -> nng_ctx {
        self.ctx
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
