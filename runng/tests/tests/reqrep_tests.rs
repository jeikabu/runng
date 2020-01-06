use crate::common::*;
use rand::Rng;
use runng::{
    asyncio::*,
    factory::latest::ProtocolFactory,
    mem,
    options::{NngOption, SetOpts},
    socket,
    socket::*,
    Error,
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread, time,
};

#[test]
fn example_basic() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let mut rep = factory.replier_open()?;
    rep.listen(&url)?;
    let mut req = factory.requester_open()?;
    req.dial(&url)?;
    req.sendmsg(NngMsg::new()?)?;
    rep.recvmsg()?;

    Ok(())
}

#[test]
fn example_async() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let mut rep_socket = factory.replier_open()?;
    rep_socket.listen(&url)?;
    let mut rep_ctx = rep_socket.create_async_stream(1)?;

    let mut req_socket = factory.requester_open()?;
    req_socket.dial(&url)?;
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

trait TestHelpers {
    fn set_timeouts(&mut self) -> runng::Result<&mut Self>;
}

impl<T> TestHelpers for T
where
    T: Socket + SetOpts,
{
    fn set_timeouts(&mut self) -> runng::Result<&mut Self> {
        self.set_duration(NngOption::SENDTIMEO, DURATION_LONG)?
            .set_duration(NngOption::RECVTIMEO, DURATION_LONG)?;
        Ok(self)
    }
}

#[test]
fn zerocopy() -> Result<(), failure::Error> {
    let url = get_url();
    let factory = ProtocolFactory::default();

    let mut rep = factory.replier_open()?;
    rep.set_timeouts()?.listen(&url)?;

    let mut req = factory.requester_open()?;
    req.set_timeouts()?.dial(&url)?;

    let start = time::Instant::now();
    let mut count = 0;
    while start.elapsed() < (DURATION_TEST / 2) {
        let mut data = mem::Alloc::with_capacity(128).unwrap();
        rand::thread_rng().fill(data.as_mut_slice());

        req.send_zerocopy(data.clone())?;
        let request = rep.recv_zerocopy()?;
        rep.send_zerocopy(request)?;
        let reply = req.recv_zerocopy()?;
        assert_eq!(data, reply);
        count += 1;
    }
    assert!(count > 1, "Received {}", count);
    Ok(())
}

type DummyMsg = u32;

fn send_loop<T>(socket: &T, start: time::Instant)
where
    T: socket::SendSocket,
{
    let dummy_message: DummyMsg = 0;
    while start.elapsed() < DURATION_TEST {
        let res = socket.send_flags(&dummy_message.to_be_bytes(), socket::Flags::NONBLOCK);
        match res {
            Ok(_) => return,
            Err(Error::Errno(NngErrno::EAGAIN)) => sleep_fast(),
            Err(err) => panic!(err),
        }
    }
    panic!("Timeout");
}

fn receive_loop<T>(socket: &T, start: time::Instant)
where
    T: socket::RecvSocket,
{
    while start.elapsed() < DURATION_TEST {
        let mut buffer = [0u8; std::mem::size_of::<DummyMsg>()];
        let recv = socket.recv_flags(&mut buffer, socket::Flags::NONBLOCK);
        match recv {
            Ok(_) => return,
            Err(Error::Errno(NngErrno::EAGAIN)) => sleep_fast(),
            Err(err) => panic!(err),
        }
    }
    panic!("Timeout");
}

#[test]
fn nonblock() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let flags = socket::SocketFlags::NONBLOCK;
    let mut rep = factory.replier_open()?;
    rep.listen_flags(&url, flags)?;
    let mut req = factory.requester_open()?;
    req.dial_flags(&url, flags)?;

    let start = time::Instant::now();
    let mut count = 0;
    while start.elapsed() < (DURATION_TEST / 2) {
        //rand::thread_rng().fill(data.as_mut_slice());
        send_loop(&req, start);
        receive_loop(&rep, start);
        send_loop(&rep, start);
        receive_loop(&req, start);
        //assert_eq!(data, reply);
        count += 1;
    }
    assert!(count > 1, "Received {}", count);
    Ok(())
}

#[test]
fn blocking() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let mut rep = factory.replier_open()?;
    rep.listen(&url)?;
    let mut req = factory.requester_open()?;
    req.dial(&url)?;

    for _ in 0..10 {
        let mut msg = vec![0u8; 128];
        rand::thread_rng().fill(msg.as_mut_slice());
        req.send(&msg)?;

        let mut buffer = vec![0u8; 1024];
        let request = rep.recv(buffer.as_mut_slice())?;
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
    let mut rep_socket = factory.replier_open()?;
    rep_socket.listen(&url)?;
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
                rep_ctx.reply(msg).then(|res| {
                    res.unwrap().unwrap();
                    future::ready(())
                })
            });
        block_on(fut);
        Ok(())
    });

    // Requesters share a socket
    let mut req_socket = factory.requester_open()?;
    req_socket.dial(&url)?;
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
