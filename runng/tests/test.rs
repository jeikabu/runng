extern crate futures;
extern crate runng;
extern crate runng_sys;

use futures::future::Future;
use runng::protocol::*;
use runng::socket::*;
use runng::*;
use runng_sys::nng_msg;
use std::{thread, time::Duration};

use std::sync::atomic::{AtomicUsize, Ordering};

static URL_ID: AtomicUsize = AtomicUsize::new(1);
fn get_url() -> String {
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}

#[test]
fn it_works() -> NngReturn {
    let url = get_url();

    let factory = Latest::new();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::new()?);
    rep.recv()?;

    Ok(())
}

#[test]
fn aio() -> NngReturn {
    let url = get_url();

    let factory = Latest::new();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;

    let requester = factory.requester_open()?.dial(&url)?;
    let mut req_ctx = requester.create_async_context()?;
    let req_future = req_ctx.send(msg::NngMsg::new()?);
    rep_ctx.receive().wait().unwrap()?;
    rep_ctx.reply(msg::NngMsg::new()?)
        .wait()
        .unwrap()?;
    req_future.wait().unwrap()?;

    Ok(())
}

#[test]
fn msg() -> NngReturn {
    let mut builder = msg::MsgBuilder::new();
    let value: u32 = 0x01234567;
    builder.append_u32(value);
    let mut msg = builder.build()?;
    assert_eq!(value, msg.trim_u32()?);

    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut msg = builder.clean().append_slice(&data).build()?;
    let mut nngmsg = msg::NngMsg::new()?;
    nngmsg.append(data.as_ptr(), data.len())?;
    assert_eq!(nngmsg.body(), msg.body());

    Ok(())
}

#[test]
fn pubsub() -> NngReturn {
    let url = get_url();
    let factory = Latest::new();

    let publisher = factory.publisher_open()?.listen(&url)?;
    let subscriber = factory.subscriber_open()?.dial(&url)?;

    let num_msg_per_subscriber = 4;

    let sub_thread = thread::spawn(move || -> NngReturn {
        let mut sub_ctx = subscriber.create_async_context()?;
        let topic: Vec<u8> = vec![0; 4];
        sub_ctx.subscribe(topic.as_slice())?;
        for _ in 0..num_msg_per_subscriber {
            sub_ctx.receive().wait().unwrap().unwrap()?;
        }
        Ok(())
    });
    let pub_thread = thread::spawn(move || -> NngReturn {
        let mut pub_ctx = publisher.create_async_context()?;

        // Beginning of message body contains topic
        let msg = msg::MsgBuilder::new()
            .append_u32(0)
            .append_u32(1)
            .build()?;

        for _ in 0..num_msg_per_subscriber {
            let msg = msg.dup()?;
            pub_ctx.send(msg).wait().unwrap()?;
            thread::sleep(Duration::from_millis(25));
        }
        Ok(())
    });

    sub_thread.join().unwrap();
    pub_thread.join().unwrap();

    Ok(())
}

#[test]
fn pushpull() -> NngReturn {
    let url = get_url();
    let factory = Latest::new();

    let pusher = factory.pusher_open()?.listen(&url)?;
    let puller = factory.puller_open()?.dial(&url)?;
    let count = 4;
    let push_thread = thread::spawn(move || -> NngReturn {
        let mut push_ctx = pusher.create_async_context()?;
        for _ in 0..count {
            push_ctx
                .send(msg::NngMsg::new()?)
                .wait()
                .unwrap()?;
        }
        Ok(())
    });
    let recv_count = AtomicUsize::new(0);
    let pull_thread = thread::spawn(move || -> NngReturn {
        let mut pull_ctx = puller.create_async_context()?;
        for _ in 0..count {
            pull_ctx.receive().wait().unwrap().unwrap()?;
            recv_count.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    });
    push_thread.join().unwrap();
    pull_thread.join().unwrap();

    Ok(())
}

#[test]
fn broker() -> NngReturn {
    let url_broker_in = get_url();
    let url_broker_out = get_url();

    let factory = Latest::new();

    let broker_pull = factory
        .puller_open()?.listen(&url_broker_in)?;
    let broker_push = factory.publisher_open()?.listen(&url_broker_out)?;

    thread::sleep(Duration::from_millis(50));

    thread::spawn(move || -> NngReturn {
        let mut broker_pull_ctx = broker_pull.create_async_context()?;
        let mut broker_push_ctx = broker_push.create_async_context()?;
        for _ in 0..10 {
            let msg = broker_pull_ctx.receive().wait().unwrap().unwrap()?;
            broker_push_ctx.send(msg).wait().unwrap()?;
        }
        Ok(())
    });

    let publisher = factory.pusher_open()?.dial(&url_broker_in)?;
    let subscriber = factory
        .subscriber_open()?
        .dial(&url_broker_out)?;

    thread::spawn(move || -> NngReturn {
        let mut sub_ctx = subscriber.create_async_context()?;

        let topic: Vec<u8> = vec![0; 4];
        sub_ctx.subscribe(topic.as_slice())?;
        for _ in 0..10 {
            let msg = sub_ctx.receive().wait().unwrap().unwrap()?;
        }
        Ok(())
    });

    thread::sleep(Duration::from_millis(50));

    thread::spawn(move || -> NngReturn {
        let mut pub_ctx = publisher.create_async_context()?;
        for _ in 0..10 {
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?;
            pub_ctx.send(msg).wait().unwrap()?;
            thread::sleep(Duration::from_millis(200));
        }
        Ok(())
    });

    thread::sleep(Duration::from_secs(3));

    Ok(())
}
