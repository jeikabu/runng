extern crate env_logger;
extern crate futures;
extern crate runng;
extern crate runng_sys;

mod common;

#[cfg(test)]
mod tests {

use common::get_url;
use futures::{
    future,
    future::Future,
    Stream,
};
use runng::{
    *,
    protocol::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

#[test]
fn pushpull() -> NngReturn {
    let url = get_url();
    let factory = Latest::new();

    let pusher = factory.pusher_open()?.listen(&url)?;
    let puller = factory.puller_open()?.dial(&url)?;
    let count = 4;

    // Pusher
    let push_thread = thread::spawn(move || -> NngReturn {
        let mut push_ctx = pusher.create_async_context()?;
        // Send messages
        for i in 0..count {
            let msg = msg::MsgBuilder::default()
                .append_u32(i).build()?;
            push_ctx
                .send(msg)
                .wait()
                .unwrap()?;
        }
        // Send a stop message
        let stop_message = msg::NngMsg::new().unwrap();
        push_ctx
            .send(stop_message)
            .wait()
            .unwrap()?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let thread_count = recv_count.clone();
    let pull_thread = thread::spawn(move || -> NngReturn {
        let mut pull_ctx = puller.create_async_context()?;
        pull_ctx.receive()
            // Process until receive stop message
            .take_while(|res| {
                match res {
                    Ok(msg) => future::ok(msg.len() > 0),
                    Err(_) => future::ok(false),
                }
            })
            // Increment count of received messages
            .for_each(|_|{
                thread_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }).wait().unwrap();
        Ok(())
    });

    push_thread.join().unwrap();
    pull_thread.join().unwrap();

    // Received number of messages we sent
    assert_eq!(recv_count.load(Ordering::Relaxed), count as usize);

    Ok(())
}

}