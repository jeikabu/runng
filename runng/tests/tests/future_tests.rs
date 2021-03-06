use crate::common::*;
use log::debug;
use runng::{asyncio::*, factory::compat::ProtocolFactory, msg::NngMsg, socket::*};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[test]
fn pushpull_timeout() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let mut pusher = factory.pair_open()?;
    pusher.listen(&url)?;

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
            let mut msg = NngMsg::new()?;
            msg.append_u32(count)?;
            block_on(push_ctx.send(msg))?;
            count += 1;
            sleep_brief();
        }
        Ok(())
    });

    // Puller
    let mut puller = factory.pair_open()?;
    puller.dial(&url)?;
    let recv_count = Arc::new(AtomicUsize::new(0));
    let lost_count = Arc::new(AtomicUsize::new(0));
    let pull_vars = (done.clone(), recv_count.clone(), lost_count.clone());
    let pull_thread = thread::spawn(move || -> runng::Result<()> {
        let (done, recv_count, lost_count) = pull_vars;
        let mut read_ctx = puller.create_async()?;
        let mut recv_msg_id = 0;
        puller_ready.store(true, Ordering::Relaxed);
        let mut prev_recv: Option<AsyncMsg> = None;
        while !done.load(Ordering::Relaxed) {
            let recv_fut = prev_recv.unwrap_or(read_ctx.receive());

            let duration = Duration::from_millis(100);
            let fut = timeout(recv_fut, duration).then(|res| match res {
                TimeoutResult::Ok(msg) => {
                    let id = msg.unwrap().trim_u32().unwrap();
                    let expect_id = recv_msg_id + 1;
                    if id != expect_id {
                        debug!("Lost a message!  Expected {}, got {}", expect_id, id);
                        lost_count.fetch_add((id - expect_id) as usize, Ordering::Relaxed);
                    }
                    recv_msg_id = id;
                    recv_count.fetch_add(1, Ordering::Relaxed);
                    future::ready(None)
                }
                TimeoutResult::Timeout(fut) => {
                    debug!("Error");
                    future::ready(Some(fut))
                }
            });
            prev_recv = block_on(fut);
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
