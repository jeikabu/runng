use crate::common::*;
use futures::future::Future;
use log::debug;
use rand::RngCore;
use runng::{
    asyncio::*,
    factory::latest::ProtocolFactory,
    msg::NngMsg,
    options::{NngOption, SetOpts},
    protocol,
    protocol::Subscribe,
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

fn create_pub(url: &str) -> runng::Result<protocol::Pub0> {
    let mut sock = protocol::Pub0::open()?;
    sock.socket_mut().set_ms(NngOption::SENDTIMEO, 100)?;
    // Pub socket doesn't support SENDBUF, messages are just dropped.
    //sock.socket_mut().set_int(NngOption::SENDBUF, 1000)?;
    sock.listen(&url)
}

fn create_sub(url: &str) -> runng::Result<protocol::Sub0> {
    let mut sock = protocol::Sub0::open()?;
    sock.socket_mut().set_ms(NngOption::RECVTIMEO, 100)?;
    // Sub socket doesn't support RECVBUF, messages are just dropped.
    //sock.socket_mut().set_int(NngOption::RECVBUF, 1000)?;
    sock.subscribe(&[]).unwrap();
    sock.dial(&url)
}

#[test]
fn bad_sub() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let sub_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());
    let pub_vars = (done.clone(), sub_ready.clone());

    // Pusher
    let pusher = create_pub(&url)?;
    let push_thread = thread::spawn(move || -> runng::Result<()> {
        let (done, sub_ready) = pub_vars;
        let mut push_ctx = pusher.create_async()?;

        // Wait for puller (or end of test)
        while !sub_ready.load(Ordering::Relaxed) && !done.load(Ordering::Relaxed) {
            sleep_brief();
        }

        // Send messages
        let mut count = 1;
        while !done.load(Ordering::Relaxed) {
            let mut msg = NngMsg::new()?;
            msg.append_u32(count)?;
            match push_ctx.send(msg).wait().unwrap() {
                // Only increment the count on success so if send fails we retry.
                Ok(_) => count += 1,
                // If get timeout just retry
                Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => {}
                err => panic!("Unexpected: {:?}", err),
            }
            // Pub/sub drops messages, so need to sleep
            sleep_brief();
        }
        Ok(())
    });

    // Puller
    let subscriber = create_sub(&url)?;
    let recv_count = Arc::new(AtomicUsize::new(0));
    let lost_count = Arc::new(AtomicUsize::new(0));
    let sub_vars = (done.clone(), recv_count.clone(), lost_count.clone());
    let sub_thread = thread::spawn(move || -> runng::Result<()> {
        let (done, recv_count, lost_count) = sub_vars;
        let mut ctx = subscriber.create_async()?;
        let mut recv_msg_id = 0;
        sub_ready.store(true, Ordering::Relaxed);
        while !done.load(Ordering::Relaxed) {
            match ctx.receive().wait() {
                Ok(Ok(mut msg)) => {
                    let id = msg.trim_u32()?;
                    debug!("recv: {}", id);
                    let expect_id = recv_msg_id + 1;
                    if id != expect_id {
                        debug!("Lost a message!  Expected {}, got {}", expect_id, id);
                        lost_count.fetch_add((id - expect_id) as usize, Ordering::Relaxed);
                        // Once the test has failed, just let it exit
                        done.store(true, Ordering::Relaxed);
                        break;
                    }
                    recv_msg_id = id;
                    recv_count.fetch_add(1, Ordering::Relaxed);
                }
                // If get read timeout loop back around and retry in case it was spurious
                Ok(Err(runng::Error::Errno(NngErrno::ETIMEDOUT))) => debug!("Read timeout"),
                err => panic!("Unexpected: {:?}", err),
            }
        }
        Ok(())
    });

    let sub_vars = (done.clone());
    let _bad_thread = thread::spawn(move || -> runng::Result<()> {
        let (done) = sub_vars;
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
    sub_thread.join().unwrap()?;

    assert!(recv_count.load(Ordering::Relaxed) > 1);
    assert_eq!(0, lost_count.load(Ordering::Relaxed));

    Ok(())
}
