use crate::common::{create_stop_message, get_url, not_stop_message};
use futures::{
    future::{Either, Future, IntoFuture},
    stream::{once, Stream},
};
use futures_timer::Delay;
use log::{debug, info};
use runng::{protocol::*, *};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

fn forward(
    ctx: &mut AsyncPairContext,
    msg: Result<msg::NngMsg, NngFail>,
) -> impl IntoFuture<Item = (), Error = ()> {
    // Increment value.  If larger than some value send a stop message and return Err to stop for_each().  Otherwise forware it.
    let mut msg = msg.unwrap();
    let value = msg.trim_u32().unwrap() + 1;
    if value > 100 {
        ctx.send(create_stop_message()).wait().unwrap();
        Err(())
    } else {
        msg.append_u32(value).unwrap();
        ctx.send(msg).wait().unwrap();
        Ok(())
    }
}

#[test]
fn pair() -> NngReturn {
    let url = get_url();
    let factory = Latest::default();
    let a = factory.pair_open()?.listen(&url)?;
    let b = factory.pair_open()?.dial(&url)?;

    let a_thread = thread::spawn(move || -> NngReturn {
        let mut ctx = a.create_async_context()?;
        // Send the first (0th) message
        let mut msg = msg::NngMsg::create()?;
        msg.append_u32(0);
        ctx.send(msg).wait()?;
        let stream = ctx
            .receive()
            .unwrap()
            .take_while(not_stop_message)
            // Receive a message and send it to other pair
            .for_each(|msg| forward(&mut ctx, msg))
            .wait()
            .unwrap();
        Ok(())
    });
    let b_thread = thread::spawn(move || -> NngReturn {
        let mut ctx = b.create_async_context()?;
        ctx.receive()
            .unwrap()
            .take_while(not_stop_message)
            // Receive a message and send it to other pair
            .for_each(|msg| forward(&mut ctx, msg))
            .wait();
        Ok(())
    });

    a_thread.join().unwrap()?;
    b_thread.join().unwrap()?;
    Ok(())
}

#[ignore]
#[test]
fn pair1_poly() -> NngReturn {
    let url = get_url();
    let factory = Latest::default();

    // Enable pair ver 1 socket "polyamorous" mode; multiple dialers can share a socket
    let mut a = factory.pair_open()?;
    a.socket_mut().setopt_bool(NngOption::PAIR1_POLY, true)?;
    let mut b = factory.pair_open()?;
    // Only listener needs PAIR1_POLY
    //b.socket_mut().setopt_bool(NngOption::PAIR1_POLY, true)?;
    b.socket_mut().setopt_ms(NngOption::SENDTIMEO, 50)?;

    let mut threads = vec![];
    {
        let url = url.clone();
        let thread = thread::spawn(move || -> NngReturn {
            let mut ctx = a.listen(&url)?.create_async_context()?;
            ctx.receive()
                .unwrap()
                .take_while(not_stop_message)
                .for_each(|msg| {
                    let msg = msg.unwrap();
                    // Duplicate message so it has same content (sender will check for identifier)
                    let mut response = msg.dup().unwrap();
                    // Response message's pipe must be set to that of the received message
                    let pipe = msg.get_pipe().unwrap();
                    response.set_pipe(&pipe);
                    ctx.send(response).wait().unwrap();
                    Ok(())
                })
                .wait();
            Ok(())
        });
        threads.push(thread);
    }

    // Let listener start
    thread::sleep(Duration::from_millis(50));

    const NUM_DIALERS: u32 = 2;
    for i in 0..NUM_DIALERS {
        let url = url.clone();
        let socket = b.clone();
        let thread = thread::spawn(move || -> NngReturn {
            let mut ctx = socket.dial(&url)?.create_async_context()?;
            // Send message containing identifier
            let msg = msg::MsgBuilder::default().append_u32(i).build()?;
            ctx.send(msg).wait().unwrap();
            // Receive reply and make sure it has same identifier
            let res = ctx
                .receive()
                .unwrap()
                .into_future()
                .select2(Delay::new(Duration::from_secs(1)))
                .then(|res| match res {
                    Ok(Either::A((msg_stream, _timeout))) => Ok(msg_stream),
                    Ok(Either::B((_timeout_error, _))) => Err(()),
                    _ => Err(()),
                })
                .wait();
            let res = if let Ok((msg, stream)) = res {
                let mut msg = msg.unwrap().unwrap();
                let reply = msg.trim_u32().unwrap();
                //TODO: this logic may not be correct.  The listener may not be able to send a reply to the sender via the pipe
                //https://github.com/nanomsg/nng/issues/862
                // if i == reply {
                //     Ok(())
                // } else {
                //     Err(())
                // }
                Ok(())
            } else {
                Err(())
            };
            ctx.send(create_stop_message()).wait()?;
            // FIXME: when reexamine Result handling, impl From should permit `into()` to be used
            if res.is_ok() {
                Ok(())
            } else {
                Err(NngFail::Unknown(-1))
            }
        });
        threads.push(thread);
    }
    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());
    Ok(())
}
