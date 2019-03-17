use crate::common::*;
use futures::{
    future::{Future, IntoFuture},
    stream::Stream,
    sync::oneshot,
};
use futures_timer::Delay;
use log::debug;
use runng::{
    asyncio::*,
    factory::latest::ProtocolFactory,
    msg::NngMsg,
    options::{NngOption, SetOpts},
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

fn forward(
    ctx: &mut PairStreamHandle,
    msg: runng::Result<NngMsg>,
) -> impl IntoFuture<Item = (), Error = ()> {
    // Increment value.  If larger than some value send a stop message and return Err to stop for_each().  Otherwise forward it.
    let mut msg = msg.unwrap();
    let value = msg.trim_u32().unwrap() + 1;
    if value > 100 {
        ctx.send(create_stop_message()).wait().unwrap().unwrap();
        Err(())
    } else {
        msg.append_u32(value).unwrap();
        ctx.send(msg).wait().unwrap().unwrap();
        Ok(())
    }
}

#[test]
fn pair() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();
    let a = factory.pair_open()?.listen(&url)?;
    let b = factory.pair_open()?.dial(&url)?;

    let a_thread = thread::spawn(move || -> runng::Result<()> {
        let mut ctx = a.create_async_stream(1)?;
        // Send the first (0th) message
        let mut msg = NngMsg::create()?;
        msg.append_u32(0)?;
        ctx.send(msg).wait()??;
        let _stream = ctx
            .receive()
            .unwrap()
            .take_while(not_stop_message)
            // Receive a message and send it to other pair
            .for_each(|msg| forward(&mut ctx, msg))
            .wait()
            .unwrap();
        Ok(())
    });
    let b_thread = thread::spawn(move || -> runng::Result<()> {
        let mut ctx = b.create_async_stream(1)?;
        ctx.receive()
            .unwrap()
            .take_while(not_stop_message)
            // Receive a message and send it to other pair
            .for_each(|msg| forward(&mut ctx, msg))
            .wait()
            .expect_err("Err means stop processing stream");
        Ok(())
    });

    a_thread.join().unwrap()?;
    b_thread.join().unwrap()?;
    Ok(())
}

//#[ignore]
#[test]
fn pair1_poly() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    // Enable pair ver 1 socket "polyamorous" mode; multiple dialers can share a socket
    let mut a = factory.pair_open()?;
    a.socket_mut().setopt_bool(NngOption::PAIR1_POLY, true)?;
    a.socket_mut().setopt_ms(NngOption::RECVTIMEO, 100)?;
    a.socket_mut().setopt_ms(NngOption::SENDTIMEO, 100)?;
    let mut b = factory.pair_open()?;
    // Only listener needs PAIR1_POLY
    //b.socket_mut().setopt_bool(NngOption::PAIR1_POLY, true)?;
    b.socket_mut().setopt_ms(NngOption::RECVTIMEO, 100)?;
    b.socket_mut().setopt_ms(NngOption::SENDTIMEO, 100)?;

    let puller_ready = Arc::new(AtomicBool::default());
    let done = Arc::new(AtomicBool::default());

    let mut threads = vec![];
    {
        let url = url.clone();
        let done = done.clone();
        let thread = thread::spawn(move || -> runng::Result<()> {
            let mut ctx = a.listen(&url)?.create_async()?;
            while !done.load(Ordering::Relaxed) {
                match ctx.receive().wait() {
                    Ok(Ok(msg)) => {
                        // Duplicate message so it has same content (sender will check for identifier)
                        let mut response = msg.dup().unwrap();
                        // Response message's pipe must be set to that of the received message
                        let pipe = msg.get_pipe().unwrap();
                        response.set_pipe(&pipe);
                        ctx.send(response).wait().unwrap().unwrap();
                    }
                    Ok(Err(runng::Error::Errno(NngErrno::ETIMEDOUT))) => break,
                    Ok(Err(err)) => panic!(err),
                    _ => break,
                }
            }
            Ok(())
        });
        threads.push(thread);
    }

    // Let listener start
    thread::sleep(Duration::from_millis(50));

    const NUM_DIALERS: u32 = 2;
    let count = Arc::new(AtomicUsize::new(0));
    for i in 0..NUM_DIALERS {
        let url = url.clone();
        let socket = b.clone();
        let count = count.clone();
        let done = done.clone();
        let thread = thread::spawn(move || -> runng::Result<()> {
            let mut ctx = socket.dial(&url)?.create_async()?;
            while !done.load(Ordering::Relaxed) {
                // Send message containing identifier
                let mut msg = NngMsg::create()?;
                msg.append_u32(i)?;
                match ctx.send(msg).wait() {
                    Ok(Ok(())) => {}
                    Ok(Err(runng::Error::Errno(NngErrno::ETIMEDOUT))) => break,
                    Ok(Err(err)) => panic!(err),
                    Err(oneshot::Canceled) => panic!(),
                }

                match ctx.receive().wait() {
                    Ok(Ok(mut msg)) => {
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
                    Ok(Err(runng::Error::Errno(NngErrno::ETIMEDOUT))) => break,
                    Ok(Err(err)) => panic!(err),
                    _ => break,
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
