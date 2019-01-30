use crate::common::{create_stop_message, get_url, not_stop_message};
use futures::{future::Future, Stream};
use log::info;
use runng::{asyncio::*, *};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

#[test]
fn example_basic() -> NngReturn {
    info!("basic");
    let url = get_url();

    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::create()?)?;
    rep.recv()?;

    Ok(())
}

#[test]
fn example_async() -> NngReturn {
    let url = get_url();

    let factory = Latest::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_stream(1)?;

    let mut req_ctx = factory.requester_open()?.dial(&url)?.create_async()?;
    let req_future = req_ctx.send(msg::NngMsg::create()?);
    rep_ctx
        .receive()
        .unwrap()
        .take(1)
        .for_each(|_request| {
            let msg = msg::NngMsg::create().unwrap();
            rep_ctx.reply(msg).wait().unwrap().unwrap();
            Ok(())
        })
        .wait()?;
    req_future.wait().unwrap()?;

    Ok(())
}

#[test]
fn contexts() -> NngReturn {
    let url = get_url();
    let factory = Latest::default();

    let recv_count = Arc::new(AtomicUsize::new(0));

    // Replier
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_stream(1)?;
    let rep_recv_count = recv_count.clone();
    let rep = thread::spawn(move || -> NngReturn {
        rep_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(not_stop_message)
            .for_each(|_request| {
                rep_recv_count.fetch_add(1, Ordering::Relaxed);

                let msg = msg::NngMsg::create().unwrap();
                rep_ctx.reply(msg).wait().unwrap().unwrap();
                Ok(())
            })
            .wait()?;
        Ok(())
    });

    // Requesters share a socket
    let req_socket = factory.requester_open()?.dial(&url)?;
    const NUM_REQUESTERS: u32 = 2;
    const NUM_REQUESTS: u32 = 100;
    let mut threads = vec![];
    for _ in 0..NUM_REQUESTERS {
        let socket_clone = req_socket.clone();
        let req = thread::spawn(move || -> NngReturn {
            let mut req_ctx = socket_clone.create_async()?;
            for i in 0..NUM_REQUESTS {
                let mut msg = msg::NngMsg::create()?;
                msg.append_u32(i)?;
                req_ctx.send(msg).wait()?;
            }
            Ok(())
        });
        threads.push(req);
    }
    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());

    // Send stop message so repier exits
    let mut req_ctx = req_socket.create_async()?;
    // Call close() to "handle" the oneshot Receiver because the replier won't actually reply
    req_ctx.send(create_stop_message()).close();
    rep.join().unwrap()?;

    assert_eq!(
        recv_count.load(Ordering::Relaxed),
        (NUM_REQUESTERS * NUM_REQUESTS) as usize
    );

    Ok(())
}
