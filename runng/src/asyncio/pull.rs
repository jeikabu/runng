//! Async read

use super::*;
use crate::{
    aio::{AioCallbackArg, NngAio},
    msg::NngMsg,
    *,
};
use futures::{future, future::Future, sync::oneshot};
use runng_sys::*;
use std::sync::Mutex;

struct PullAioArg {
    aio: NngAio,
    queue: Mutex<WorkQueue>,
}

impl PullAioArg {
    pub fn create(socket: NngSocket) -> NngResult<Box<Self>> {
        let aio = NngAio::new(socket);
        let queue = Mutex::new(WorkQueue::default());
        let arg = Self { aio, queue };
        let context = NngAio::register_aio(arg, read_callback);
        if let Ok(ref context) = context {
            context.receive();
        }
        context
    }

    fn receive(&self) {
        unsafe {
            nng_recv_aio(self.aio.nng_socket(), self.aio.nng_aio());
        }
    }
}

impl Aio for PullAioArg {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

pub struct PullAsyncHandle {
    aio_arg: Box<PullAioArg>,
}

impl AsyncContext for PullAsyncHandle {
    fn create(socket: NngSocket) -> NngResult<Self> {
        let aio_arg = PullAioArg::create(socket)?;
        Ok(Self { aio_arg })
    }
}

pub trait ReadAsync {
    // FIXME: Can change this to -> impl Future later?
    fn receive(&mut self) -> Box<dyn Future<Item = NngResult<NngMsg>, Error = oneshot::Canceled>>;
}

impl ReadAsync for PullAsyncHandle {
    fn receive(&mut self) -> Box<dyn Future<Item = NngResult<NngMsg>, Error = oneshot::Canceled>> {
        let mut queue = self.aio_arg.queue.lock().unwrap();
        // If a value is ready return it immediately.  Otherwise
        if let Some(item) = queue.ready.pop_front() {
            Box::new(future::ok(item))
        } else {
            let (sender, receiver) = oneshot::channel();
            queue.waiting.push_back(sender);
            Box::new(receiver)
        }
    }
}

unsafe extern "C" fn read_callback(arg: AioCallbackArg) {
    let ctx = &mut *(arg as *mut PullAioArg);
    let aio = ctx.aio.nng_aio();
    let aio_res = nng_aio_result(aio);
    let res = NngFail::from_i32(aio_res);
    trace!("read_callback::{:?}", res);
    match res {
        Err(res) => {
            match res {
                // nng_aio_close() calls nng_aio_stop which nng_aio_abort(NNG_ECANCELED) and waits.
                // If we call start_receive() it will fail with ECANCELED and we infinite loop...
                NngFail::Err(nng_errno_enum::NNG_ECLOSED)
                | NngFail::Err(nng_errno_enum::NNG_ECANCELED) => {
                    debug!("read_callback {:?}", res);
                }
                _ => {
                    trace!("read_callback::Err({:?})", res);
                    ctx.receive();
                }
            }
            ctx.queue.lock().unwrap().push_back(Err(res));
        }
        Ok(()) => {
            let msg = NngMsg::new_msg(nng_aio_get_msg(aio));
            ctx.receive();
            ctx.queue.lock().unwrap().push_back(Ok(msg));
        }
    }
}
