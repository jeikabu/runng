use crate::common::*;
use runng::{
    asyncio::*,
    options::{NngOption, SetOpts},
    socket::*,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

fn create_pusher(url: &str) -> runng::Result<protocol::Push0> {
    let mut sock = protocol::Push0::open()?;
    sock.set_duration(NngOption::SENDTIMEO, DURATION_LONG)?
        .set_int(NngOption::SENDBUF, 1000)?;
    sock.listen(&url)?;
    Ok(sock)
}

fn create_puller(url: &str) -> runng::Result<protocol::Pull0> {
    let mut sock = protocol::Pull0::open()?;
    sock.set_duration(NngOption::RECVTIMEO, DURATION_LONG)?
        .set_int(NngOption::RECVBUF, 1000)?;
    sock.dial(&url)?;
    Ok(sock)
}

#[test]
fn pull_stream() -> runng::Result<()> {
    let url = get_url();
    let pusher = create_pusher(&url)?;
    let puller = create_puller(&url)?;

    let puller_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());

    // Pusher
    let push_vars = (puller_ready.clone(), done.clone());
    let push_thread = thread::spawn(move || -> runng::Result<()> {
        let mut push_ctx = pusher.create_async()?;
        let (puller_ready, done) = push_vars;

        // Wait for puller (or end of test)
        while !puller_ready.load(Ordering::Relaxed) && !done.load(Ordering::Relaxed) {
            sleep_brief();
        }

        let mut count = 1;
        while !done.load(Ordering::Relaxed) {
            let mut msg = NngMsg::new()?;
            msg.append_u32(count)?;
            count += 1;
            block_on(push_ctx.send(msg))?;
            sleep_brief();
        }
        // Send a stop message
        block_on(push_ctx.send(create_stop_message()))?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let pull_vars = (puller_ready.clone(), recv_count.clone());
    let pull_thread = thread::spawn(move || -> runng::Result<()> {
        let mut pull_ctx = puller.create_async_stream(1)?;
        let (puller_ready, recv_count) = pull_vars;
        puller_ready.store(true, Ordering::Relaxed);
        let fut = pull_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(not_stop_message)
            // Increment count of received messages
            .for_each(|_| {
                recv_count.fetch_add(1, Ordering::Relaxed);
                future::ready(())
            });
        block_on(fut);
        Ok(())
    });

    sleep_test();
    done.store(true, Ordering::Relaxed);
    push_thread.join().unwrap()?;
    pull_thread.join().unwrap()?;
    assert!(recv_count.load(Ordering::Relaxed) > 1);

    Ok(())
}

#[test]
fn read() -> runng::Result<()> {
    let url = get_url();

    let pusher = create_pusher(&url)?;
    let puller = create_puller(&url)?;
    let puller_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());

    // Pusher
    let push_vars = (puller_ready.clone(), done.clone());
    let push_thread = thread::spawn(move || -> runng::Result<()> {
        let (puller_ready, done) = push_vars;
        let mut push_ctx = pusher.create_async()?;

        // Wait for puller (or end of test)
        while !puller_ready.load(Ordering::Relaxed) && !done.load(Ordering::Relaxed) {
            sleep_brief();
        }

        // Send messages
        let mut count = 1;
        while !done.load(Ordering::Relaxed) {
            let mut msg = NngMsg::new()?;
            msg.append_u32(count)?;
            count += 1;
            block_on(push_ctx.send(msg))?;
            sleep_brief();
        }
        // Send a stop message
        block_on(push_ctx.send(create_stop_message()))?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let pull_vars = (puller_ready.clone(), done.clone(), recv_count.clone());
    let pull_thread = thread::spawn(move || -> runng::Result<()> {
        let (puller_ready, done, thread_count) = pull_vars;
        let mut read_ctx = puller.create_async()?;
        puller_ready.store(true, Ordering::Relaxed);
        while !done.load(Ordering::Relaxed) {
            let msg = block_on(read_ctx.receive())?;
            if msg.is_empty() {
                break;
            } else {
                thread_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    });

    sleep_test();
    done.store(true, Ordering::Relaxed);

    push_thread.join().unwrap()?;
    pull_thread.join().unwrap()?;

    // Received number of messages we sent
    assert!(recv_count.load(Ordering::Relaxed) > 1);

    Ok(())
}
