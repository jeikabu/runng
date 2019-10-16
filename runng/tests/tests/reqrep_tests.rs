use crate::common::*;
use log::info;
use rand::Rng;
use runng::{asyncio::*, factory::latest::ProtocolFactory, mem, socket, socket::*, Error};
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
    req.sendmsg(NngMsg::new()?)?;
    rep.recvmsg()?;

    Ok(())
}

#[test]
fn example_async() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep_socket = factory.replier_open()?.listen(&url)?;
    let mut rep_ctx = rep_socket.create_async_stream(1)?;

    let req_socket = factory.requester_open()?.dial(&url)?;
    let mut req_ctx = req_socket.create_async()?;
    let req_future = req_ctx.send(NngMsg::new()?);
    let fut = rep_ctx.receive().unwrap().take(1).for_each(|_request| {
        let msg = NngMsg::new().unwrap();
        rep_ctx.reply(msg).then(|res| {
            res.unwrap().unwrap();
            future::ready(())
        })
    });
    block_on(fut);
    block_on(req_future).unwrap()?;

    Ok(())
}

#[test]
fn zerocopy() -> Result<(), failure::Error> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;

    for _ in 0..10 {
        let mut data = mem::Alloc::with_capacity(128).unwrap();
        rand::thread_rng().fill(data.as_mut_slice());

        req.send_zerocopy(data.clone())?;
        let request = rep.recv_zerocopy()?;
        rep.send_zerocopy(request)?;
        let reply = req.recv_zerocopy()?;
        assert_eq!(data, reply);
    }

    Ok(())
}

fn send_loop<T>(socket: &T, mut msg: NngMsg)
where
    T: socket::SendSocket,
{
    loop {
        let res = socket.sendmsg_flags(msg, socket::Flags::NONBLOCK);
        if let Err(senderror) = res {
            if senderror.error == Error::Errno(NngErrno::EAGAIN) {
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
    T: socket::RecvSocket,
{
    loop {
        let flags = socket::Flags::NONBLOCK;
        let msg = socket.recvmsg_flags(flags);
        match msg {
            Ok(msg) => return msg,
            Err(Error::Errno(NngErrno::EAGAIN)) => sleep_fast(),
            Err(err) => panic!(err),
        }
    }
}

#[test]
fn nonblock() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let flags = socket::SocketFlags::NONBLOCK;
    let rep = factory.replier_open()?.listen_flags(&url, flags)?;
    let req = factory.requester_open()?.dial_flags(&url, flags)?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    for _ in 0..10 {
        let msg = NngMsg::new()?;
        //rand::thread_rng().fill(data.as_mut_slice());
        send_loop(&req, msg);
        let request = receive_loop(&rep);
        req.recvmsg_flags(socket::Flags::NONBLOCK);
        send_loop(&rep, request);
        let _reply = receive_loop(&req);
        //assert_eq!(data, reply);
    }

    Ok(())
}

#[test]
fn blocking() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;

    for _ in 0..10 {
        let mut msg = vec![0u8; 128];
        rand::thread_rng().fill(msg.as_mut_slice());
        req.send(&mut msg)?;

        let mut buffer = vec![0u8; 1024];
        let mut request = rep.recv(buffer.as_mut_slice())?;
        rep.send(request)?;
        let mut reply = vec![0u8; 1024];
        let reply = req.recv(reply.as_mut_slice())?;
        assert_eq!(msg, reply);
    }

    Ok(())
}

#[test]
fn contexts() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let recv_count = Arc::new(AtomicUsize::new(0));

    // Replier
    let rep_socket = factory.replier_open()?.listen(&url)?;
    let mut rep_ctx = rep_socket.create_async_stream(1)?;
    let rep_recv_count = recv_count.clone();
    let rep = thread::spawn(move || -> runng::Result<()> {
        let fut = rep_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(not_stop_message)
            .for_each(|_request| {
                rep_recv_count.fetch_add(1, Ordering::Relaxed);

                let msg = NngMsg::new().unwrap();
                block_on(rep_ctx.reply(msg)).unwrap().unwrap();
                future::ready(())
            });
        block_on(fut);
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
                let mut msg = NngMsg::new()?;
                msg.append_u32(i)?;
                block_on(req_ctx.send(msg))??;
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
