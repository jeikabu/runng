use crate::common::*;
use futures::{
    future::{Future, IntoFuture},
    Stream,
};
use log::debug;
use rand::RngCore;
use runng::{
    asyncio::*,
    factory::latest::ProtocolFactory,
    msg::NngMsg,
    options::{NngOption, SetOpts},
    protocol,
    socket::*,
    NngErrno,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

fn create_pusher(url: &str) -> runng::Result<protocol::Push0> {
    let mut sock = protocol::Push0::open()?;
    sock.socket_mut().setopt_ms(NngOption::RECVTIMEO, 100)?;
    sock.socket_mut().setopt_ms(NngOption::SENDTIMEO, 100)?;
    sock.listen(&url)
}

fn create_puller(url: &str) -> runng::Result<protocol::Pull0> {
    let mut sock = protocol::Pull0::open()?;
    sock.socket_mut().setopt_ms(NngOption::RECVTIMEO, 100)?;
    sock.socket_mut().setopt_ms(NngOption::SENDTIMEO, 100)?;
    sock.dial(&url)
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
            let mut msg = NngMsg::create()?;
            msg.append_u32(count)?;
            count += 1;
            push_ctx.send(msg).wait().unwrap()?;
            sleep_brief();
        }
        // Send a stop message
        push_ctx.send(create_stop_message()).wait().unwrap()?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let pull_vars = (puller_ready.clone(), recv_count.clone());
    let pull_thread = thread::spawn(move || -> runng::Result<()> {
        let mut pull_ctx = puller.create_async_stream(1)?;
        let (puller_ready, recv_count) = pull_vars;
        puller_ready.store(true, Ordering::Relaxed);
        pull_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(not_stop_message)
            // Increment count of received messages
            .for_each(|_| {
                recv_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .wait()?;
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
    let factory = ProtocolFactory::default();

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
            let mut msg = NngMsg::create()?;
            msg.append_u32(count)?;
            count += 1;
            push_ctx.send(msg).wait().unwrap()?;
            sleep_brief();
        }
        // Send a stop message
        push_ctx.send(create_stop_message()).wait().unwrap()?;
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
            let msg = read_ctx.receive().wait()??;
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

#[test]
fn bad_puller() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let pusher = create_pusher(&url)?;

    let puller_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());
    let push_vars = (done.clone(), puller_ready.clone());

    // Pusher
    let push_thread = thread::spawn(move || -> runng::Result<()> {
        let (done, puller_ready) = push_vars;
        let mut push_ctx = pusher.create_async()?;

        // Wait for puller (or end of test)
        while !puller_ready.load(Ordering::Relaxed) && !done.load(Ordering::Relaxed) {
            sleep_brief();
        }

        // Send messages
        let mut count = 1;
        while !done.load(Ordering::Relaxed) {
            let mut msg = NngMsg::create()?;
            msg.append_u32(count)?;
            push_ctx.send(msg).wait().unwrap()?;
            count += 1;
            sleep_brief();
        }
        Ok(())
    });

    // Puller
    let puller = create_puller(&url)?;
    let recv_count = Arc::new(AtomicUsize::new(0));
    let lost_count = Arc::new(AtomicUsize::new(0));
    let pull_vars = (done.clone(), recv_count.clone(), lost_count.clone());
    let pull_thread = thread::spawn(move || -> runng::Result<()> {
        let (done, recv_count, lost_count) = pull_vars;
        let mut ctx = puller.create_async()?;
        let mut recv_msg_id = 0;
        puller_ready.store(true, Ordering::Relaxed);
        while !done.load(Ordering::Relaxed) {
            match ctx.receive().wait() {
                Ok(Ok(mut msg)) => {
                    let id = msg.trim_u32()?;
                    let expect_id = recv_msg_id + 1;
                    if id != expect_id {
                        debug!("Lost a message!  Expected {}, got {}", expect_id, id);
                        lost_count.fetch_add((id - expect_id) as usize, Ordering::Relaxed);
                    }
                    recv_msg_id = id;
                    recv_count.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Err(runng::Error::Errno(NngErrno::ETIMEDOUT))) => break,
                Ok(Err(err)) => panic!(err),
                _ => break,
            }
        }
        Ok(())
    });

    let pull_vars = (done.clone());
    let _bad_thread = thread::spawn(move || -> runng::Result<()> {
        let (done) = pull_vars;
        while !done.load(Ordering::Relaxed) {
            let puller = factory.puller_open()?.dial(&url)?;
            let mut read_ctx = puller.create_async()?;
            let recv_future = read_ctx.receive();
            let rand_sleep = ((rand::thread_rng().next_u64() & 0x7) + 1) * 2;
            thread::sleep(Duration::from_millis(rand_sleep));
        }
        Ok(())
    });

    sleep_test();
    done.store(true, Ordering::Relaxed);

    push_thread.join().unwrap()?;
    pull_thread.join().unwrap()?;

    assert!(recv_count.load(Ordering::Relaxed) > 1);
    assert_eq!(0, lost_count.load(Ordering::Relaxed));

    Ok(())
}
