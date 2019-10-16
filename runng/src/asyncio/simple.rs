//! Simple queue of asynchronous I/O work.

use super::*;
use log::debug;
use std::{collections::VecDeque, marker::PhantomPinned, pin::Pin, sync::Mutex};

/// Nng asynchronous I/O handle and queue of work items.
pub struct SimpleAioWorkQueue {
    worker: Pin<Box<AioQueue>>,
}

impl SimpleAioWorkQueue {
    pub fn new() -> Result<Self> {
        let worker = AioQueue::new()?;
        Ok(Self { worker })
    }
}

impl AioWorkQueue for SimpleAioWorkQueue {
    fn push_back(&mut self, obj: AioWorkRequest) {
        // Get mutable reference to pinned struct
        let inner: &mut _ = unsafe {
            let mut_ref = Pin::as_mut(&mut self.worker);
            Pin::get_unchecked_mut(mut_ref)
        };
        let mut shared = inner.shared.lock().unwrap();
        match shared.state {
            State::Idle => {
                shared.state = State::Busy;
                obj.begin(&inner.aio);
            }
            State::Busy => {}
        }
        shared.queue.push_back(obj);
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    Busy,
}

impl Default for State {
    fn default() -> Self {
        State::Idle
    }
}

#[derive(Default)]
struct SharedQueueData {
    state: State,
    queue: VecDeque<AioWorkRequest>,
}

struct AioQueue {
    aio: NngAio,
    shared: Mutex<SharedQueueData>,
    _phantom: PhantomPinned,
}

impl AioQueue {
    fn new_with_aio(aio: NngAio) -> AioQueue {
        Self {
            aio,
            shared: Default::default(),
            _phantom: PhantomPinned,
        }
    }

    pub fn new() -> Result<AioArg<AioQueue>> {
        NngAio::new(Self::new_with_aio, native_callback)
    }

    fn callback(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        let front = shared.queue.pop_front();
        if shared.state == State::Idle || front.is_none() {
            let res = unsafe { nng_int_to_result(nng_aio_result(self.aio.nng_aio())) };
            debug!("Unexpected callback: {:?}", res);
        } else {
            let mut front = front.unwrap();
            front.finish(self.aio());
            if let Some(next) = shared.queue.front() {
                next.begin(self.aio());
            } else {
                shared.state = State::Idle;
            }
        }
    }
}

impl Aio for AioQueue {
    fn aio(&self) -> &NngAio {
        &self.aio
    }
    fn aio_mut(&mut self) -> &mut NngAio {
        &mut self.aio
    }
}

impl std::fmt::Display for AioQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AioQueue {:?}", self.aio)
    }
}

unsafe extern "C" fn native_callback(arg: AioArgPtr) {
    let ctx = &mut *(arg as *mut AioQueue);
    ctx.callback();
}
