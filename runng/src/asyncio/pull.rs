//! Async read.

use super::*;
use crate::{msg::NngMsg, *};
use futures::{future, future::Future, sync::oneshot};
use runng_sys::*;
use std::sync::Mutex;

#[derive(Debug)]
struct PullAioArg {
    aio: NngAio,
    queue: Mutex<WorkQueue>,
    socket: NngSocket,
}

impl PullAioArg {
    pub fn new(socket: NngSocket) -> Result<AioArg<Self>> {
        let queue = Mutex::new(WorkQueue::default());
        let context = NngAio::new(|aio| Self { aio, queue, socket }, read_callback)?;
        context.receive();
        Ok(context)
    }

    fn receive(&self) {
        unsafe {
            nng_recv_aio(self.socket.nng_socket(), self.aio.nng_aio());
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

/// Async pull context for push/pull pattern.
#[derive(Debug)]
pub struct PullAsyncHandle {
    aio_arg: AioArg<PullAioArg>,
}

impl AsyncContext for PullAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let aio_arg = PullAioArg::new(socket)?;
        Ok(Self { aio_arg })
    }
}

pub trait ReadAsync {
    // FIXME: Can change this to -> impl Future later?
    fn receive(&mut self) -> Box<dyn Future<Item = Result<NngMsg>, Error = oneshot::Canceled>>;
}

impl ReadAsync for PullAsyncHandle {
    fn receive(&mut self) -> Box<dyn Future<Item = Result<NngMsg>, Error = oneshot::Canceled>> {
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

/// Asynchronous context for subscribe socket.
#[derive(Debug)]
pub struct SubscribeAsyncHandle {
    ctx: PullAsyncHandle,
}

impl AsyncContext for SubscribeAsyncHandle {
    fn new(socket: NngSocket) -> Result<Self> {
        let ctx = PullAsyncHandle::new(socket)?;
        Ok(Self { ctx })
    }
}

impl ReadAsync for SubscribeAsyncHandle {
    fn receive(&mut self) -> Box<dyn Future<Item = Result<NngMsg>, Error = oneshot::Canceled>> {
        self.ctx.receive()
    }
}

unsafe extern "C" fn read_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut PullAioArg);
    let aio = ctx.aio.nng_aio();
    let aio_res = nng_aio_result(aio);
    let res = nng_int_to_result(aio_res);
    trace!("read_callback::{:?}", res);
    match res {
        Err(res) => {
            match res {
                // nng_aio_close() calls nng_aio_stop which nng_aio_abort(NNG_ECANCELED) and waits.
                // If we call start_receive() it will fail with ECANCELED and we infinite loop...
                Error::Errno(NngErrno::ECLOSED) | Error::Errno(NngErrno::ECANCELED) => {
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
            let msg = NngMsg::from_raw(nng_aio_get_msg(aio));
            ctx.queue.lock().unwrap().push_back(Ok(msg));
            // Don't start next read until after notifying this one is complete.
            ctx.receive();
        }
    }
}
