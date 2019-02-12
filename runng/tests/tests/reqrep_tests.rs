use crate::common::{create_stop_message, get_url, not_stop_message, sleep_fast};
use futures::{future::Future, Stream};
use log::info;
use rand::Rng;
use runng::{asyncio::*, factory::latest::ProtocolFactory, memory, msg::NngMsg, socket, socket::*};
use runng_sys::*;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

#[test]
fn example_basic() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.sendmsg(NngMsg::create()?)?;
    rep.recv()?;

    Ok(())
}

#[test]
fn example_async() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_stream(1)?;

    let mut req_ctx = factory.requester_open()?.dial(&url)?.create_async()?;
    let req_future = req_ctx.send(NngMsg::create()?);
    rep_ctx
        .receive()
        .unwrap()
        .take(1)
        .for_each(|_request| {
            let msg = NngMsg::create().unwrap();
            rep_ctx.reply(msg).wait().unwrap().unwrap();
            Ok(())
        })
        .wait()?;
    req_future.wait().unwrap()?;

    Ok(())
}

#[test]
fn zerocopy() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;

    for _ in 0..10 {
        let mut data = memory::Alloc::create(128).unwrap();
        rand::thread_rng().fill(data.as_mut_slice());

        req.send_zerocopy(data.clone()).unwrap();
        let request = rep.recv_zerocopy().unwrap();
        rep.send_zerocopy(request).unwrap();
        let reply = req.recv_zerocopy().unwrap();
        assert_eq!(data, reply);
    }

    Ok(())
}

fn send_loop<T>(socket: &T, mut msg: NngMsg)
where
    T: socket::SendMsg,
{
    loop {
        let res = socket.sendmsg_flags(msg, socket::Flags::NONBLOCK);
        if let Err(senderror) = res {
            if senderror.error == nng_errno_enum::NNG_EAGAIN {
                msg = senderror.into_inner();
                sleep_fast();
            } else {
                panic!(senderror)
            }
        } else {
            break;
        }
    }
}

fn receive_loop<T>(socket: &T) -> NngMsg
where
    T: socket::RecvMsg,
{
    loop {
        let flags = socket::Flags::NONBLOCK;
        let msg = socket.recvmsg_flags(flags);
        match msg {
            Ok(msg) => return msg,
            Err(runng::Error::Errno(nng_errno_enum::NNG_EAGAIN)) => sleep_fast(),
            Err(err) => panic!(err),
        }
    }
}

#[test]
fn nonblock() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let flags = socket::Flags::NONBLOCK;
    let rep = factory.replier_open()?.listen_flags(&url, flags)?;
    let req = factory.requester_open()?.dial_flags(&url, flags)?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    for _ in 0..10 {
        let msg = NngMsg::create()?;
        //rand::thread_rng().fill(data.as_mut_slice());
        send_loop(&req, msg);
        let request = receive_loop(&rep);
        req.recvmsg_flags(socket::Flags::NONBLOCK);
        send_loop(&rep, request);
        let reply = receive_loop(&req);
        //assert_eq!(data, reply);
    }

    Ok(())
}

#[test]
fn contexts() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let recv_count = Arc::new(AtomicUsize::new(0));

    // Replier
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_stream(1)?;
    let rep_recv_count = recv_count.clone();
    let rep = thread::spawn(move || -> runng::Result<()> {
        rep_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(not_stop_message)
            .for_each(|_request| {
                rep_recv_count.fetch_add(1, Ordering::Relaxed);

                let msg = NngMsg::create().unwrap();
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
        let req = thread::spawn(move || -> runng::Result<()> {
            let mut req_ctx = socket_clone.create_async()?;
            for i in 0..NUM_REQUESTS {
                let mut msg = NngMsg::create()?;
                msg.append_u32(i)?;
                req_ctx.send(msg).wait()??;
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
