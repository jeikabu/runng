use crate::common::*;
use failure::Error;
use runng::{
    asyncio::*,
    options::{NngOption, SetOpts},
    protocol::*,
    socket::*,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

#[test]
fn simple() -> Result<(), Error> {
    init_logging();
    let in_url = get_url();
    let out_url = get_url();

    let mut broker_in = Pull0::open()?;
    broker_in.set_ms(NngOption::RECVTIMEO, 100)?.listen(&in_url)?;
    let mut broker_out = Push0::open()?;
    broker_out.listen(&out_url)?;

    let done = Arc::new(AtomicBool::default());
    let forwarded_count = Arc::new(AtomicUsize::new(0));
    let send_count = Arc::new(AtomicUsize::new(0));
    let recv_count = Arc::new(AtomicUsize::new(0));

    let broker_args = (done.clone(), forwarded_count.clone());
    let broker_thread = thread::spawn(move || -> Result<(), Error> {
        let (done, forwarded_count) = broker_args;
        let mut broker_in = broker_in.create_async()?;
        let mut broker_out = broker_out.create_async()?;
        while !done.load(Ordering::Relaxed) {
            match block_on(broker_in.receive()) {
                Ok(msg) => {
                    forwarded_count.fetch_add(1, Ordering::Relaxed);
                    block_on(broker_out.send(msg)).unwrap();
                }
                Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => break,
                Err(err) => panic!(err),
            }
        }

        Ok(())
    });

    let server_args = (done.clone(), send_count.clone());
    let server_thread = thread::spawn(move || -> Result<(), Error> {
        let (done, send_count) = server_args;
        let mut ctx = Push0::open()?.dial(&in_url)?.create_async()?;
        while !done.load(Ordering::Relaxed) {
            let msg = NngMsg::new()?;
            block_on(ctx.send(msg))?;
            send_count.fetch_add(1, Ordering::Relaxed);
            sleep_brief();
        }
        Ok(())
    });

    let client_args = (done.clone(), recv_count.clone());
    let client_thread = thread::spawn(move || -> Result<(), Error> {
        let (done, recv_count) = client_args;
        let mut socket = Pull0::open()?;
        socket
            .set_ms(NngOption::RECVTIMEO, 100)?
            .dial(&out_url)?;
        let mut ctx = socket.create_async()?;
        while !done.load(Ordering::Relaxed) {
            match block_on(ctx.receive()) {
                Ok(mut _msg) => {
                    recv_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => break,
                Err(err) => panic!(err),
            }
        }
        Ok(())
    });

    sleep_test();
    done.store(true, Ordering::Relaxed);
    server_thread.join().unwrap()?;
    client_thread.join().unwrap()?;
    broker_thread.join().unwrap()?;

    let send_count = send_count.load(Ordering::Relaxed);
    let forwarded_count = forwarded_count.load(Ordering::Relaxed);
    let recv_count = recv_count.load(Ordering::Relaxed);
    assert!(send_count > 1);
    assert!(recv_count > 1);
    assert!(send_count >= forwarded_count && forwarded_count >= recv_count);

    Ok(())
}
