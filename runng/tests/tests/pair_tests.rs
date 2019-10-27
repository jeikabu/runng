use crate::common::*;
use runng::{
    asyncio::*,
    factory::latest::ProtocolFactory,
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

// fn forward(ctx: &mut PairStreamHandle, msg: runng::Result<NngMsg>) -> AsyncUnit {
//     // Increment value.  If larger than some value send a stop message and return Err to stop for_each().  Otherwise forward it.
//     let mut msg = msg.unwrap();
//     let value = msg.trim_u32().unwrap() + 1;
//     if value > 100 {
//         ctx.send(create_stop_message())
//     } else {
//         msg.append_u32(value).unwrap();
//         ctx.send(msg)
//     }
// }

#[test]
fn pair() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();
    let mut a = factory.pair_open()?;
    a.listen(&url)?;
    let mut b = factory.pair_open()?;
    b.dial(&url)?;

    let a_thread = thread::spawn(move || -> runng::Result<()> {
        let mut ctx = a.create_async()?;
        let msg = NngMsg::new()?;
        block_on(ctx.send(msg))?;
        Ok(())
    });
    let b_thread = thread::spawn(move || -> runng::Result<()> {
        let mut ctx = b.create_async()?;
        block_on(ctx.receive())?;
        Ok(())
    });

    a_thread.join().unwrap()?;
    b_thread.join().unwrap()?;
    Ok(())
}

#[test]
fn pair1_poly() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    // Enable pair ver 1 socket "polyamorous" mode; multiple dialers can share a socket
    let mut a = factory.pair_open()?;
    a.set_bool(NngOption::PAIR1_POLY, true)?
        .set_ms(NngOption::RECVTIMEO, 100)?
        .set_ms(NngOption::SENDTIMEO, 100)?;
    let mut b = factory.pair_open()?;
    // Only listener needs PAIR1_POLY
    //b.socket_mut().set_bool(NngOption::PAIR1_POLY, true)?;
    b.set_ms(NngOption::RECVTIMEO, 100)?
        .set_ms(NngOption::SENDTIMEO, 100)?;

    let listener_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());

    let mut threads = vec![];
    {
        let url = url.clone();
        let listener_ready = listener_ready.clone();
        let done = done.clone();
        let thread = thread::spawn(move || -> runng::Result<()> {
            let mut ctx = a.listen(&url)?.create_async()?;
            listener_ready.store(true, Ordering::Relaxed);
            while !done.load(Ordering::Relaxed) {
                match block_on(ctx.receive()) {
                    Ok(msg) => {
                        // Duplicate message so it has same content (sender will check for identifier)
                        let mut response = msg.dup()?;
                        // Response message's pipe must be set to that of the received message
                        if let Some(pipe) = msg.get_pipe() {
                            response.set_pipe(&pipe);
                            block_on(ctx.send(response))?;
                        }
                    }
                    Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => break,
                    Err(err) => panic!(err),
                }
            }
            Ok(())
        });
        threads.push(thread);
    }

    while !listener_ready.load(Ordering::Relaxed) && !done.load(Ordering::Relaxed) {
        sleep_fast();
    }

    const NUM_DIALERS: u32 = 2;
    let count = Arc::new(AtomicUsize::new(0));
    for i in 0..NUM_DIALERS {
        let url = url.clone();
        let mut socket = b.clone();
        let count = count.clone();
        let done = done.clone();
        let thread = thread::spawn(move || -> runng::Result<()> {
            let mut ctx = socket.dial(&url)?.create_async()?;
            while !done.load(Ordering::Relaxed) {
                // Send message containing identifier
                let mut msg = NngMsg::new()?;
                msg.append_u32(i)?;
                match block_on(ctx.send(msg)) {
                    Ok(()) => {}
                    Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => break,
                    Err(err) => panic!(err),
                }

                match block_on(ctx.receive()) {
                    Ok(mut msg) => {
                        let _reply = msg.trim_u32()?;
                        count.fetch_add(1, Ordering::Relaxed);

                        //TODO: this logic may not be correct.  The listener may not be able to send a reply to the sender via the pipe
                        //https://github.com/nanomsg/nng/issues/862
                        // if i == reply {
                        //     Ok(())
                        // } else {
                        //     Err(())
                        // }
                    }
                    Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => break,
                    Err(err) => panic!(err),
                }
            }

            Ok(())
        });
        threads.push(thread);
    }

    sleep_test();
    done.store(true, Ordering::Relaxed);

    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());
    assert!(count.load(Ordering::Relaxed) > NUM_DIALERS as usize);
    Ok(())
}

#[test]
fn pair_stream() -> runng::Result<()> {
    Ok(())
}
