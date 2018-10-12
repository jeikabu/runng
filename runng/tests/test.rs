extern crate futures;
extern crate runng;
extern crate runng_sys;

use futures::future::Future;
use runng::*;
use runng_sys::nng_msg;
use runng::protocol::*;
use runng::socket::*;
use std::{
    thread,
    time::Duration
};

use std::sync::atomic::{AtomicUsize, Ordering};

static URL_ID: AtomicUsize = AtomicUsize::new(1);
fn get_url() -> String {
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}

#[test]
fn it_works() {
    let url = get_url();

    let factory = Latest::new();
    let req = factory.requester_open().unwrap();
    let rep = factory.replier_open().unwrap();
    rep.listen(&url).unwrap();
    req.dial(&url).unwrap();
    req.send().unwrap();
    rep.recv().unwrap();
}

#[test]
fn aio() {
    let url = get_url();

    let factory = Latest::new();
    let replier = factory.replier_open().unwrap();
    replier.listen(&url).unwrap();
    let mut rep_ctx = replier.create_async_context().unwrap();

    let requester = factory.requester_open().unwrap();
    requester.dial(&url).unwrap();
    let mut req_ctx = requester.create_async_context().unwrap();
    let req_future = req_ctx.send(msg::NngMsg::new().unwrap());
    rep_ctx.receive().wait().unwrap().unwrap();
    rep_ctx.reply(msg::NngMsg::new().unwrap()).wait().unwrap().unwrap();
    req_future.wait().unwrap().unwrap();
}

#[test]
fn msg() {
    let mut builder = msg::MsgBuilder::new();
    let value: u32 = 0x01234567;
    builder.append_u32(value);
    let mut msg = builder.build().unwrap();
    assert_eq!(value, msg.trim_u32().unwrap());

    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut msg = builder.clean().append_slice(&data).build().unwrap();
    let mut nngmsg = msg::NngMsg::new().unwrap();
    nngmsg.append(data.as_ptr() as *const ::std::os::raw::c_void, data.len()).unwrap();
    assert_eq!(nngmsg.body(), msg.body());
}

#[test]
fn pubsub() {
    let url = get_url();
    let factory = Latest::new();

    let publisher = factory.publisher_open().unwrap();
    publisher.listen(&url).unwrap();
    let subscriber = factory.subscriber_open().unwrap();
    subscriber.dial(&url).unwrap();

    let num_msg_per_subscriber = 4;

    let sub_thread = thread::spawn(move || {
        let mut sub_ctx = subscriber.create_async_context().unwrap();
        let topic: Vec<u8> = vec![0; 4];
        sub_ctx.subscribe(topic.as_slice()).unwrap();
        for _ in 0..num_msg_per_subscriber {
            sub_ctx.receive().wait().unwrap().unwrap().unwrap();
        }
    });
    let pub_thread = thread::spawn(move || {
        let mut pub_ctx = publisher.create_async_context().unwrap();
        
        // Beginning of message body contains topic
        let msg = msg::MsgBuilder::new().append_u32(0).append_u32(1).build().unwrap();

        for _ in 0..num_msg_per_subscriber {
            let msg = msg.dup().unwrap();
            pub_ctx.send(msg).wait().unwrap().unwrap();
            thread::sleep(Duration::from_millis(25));
        }
    });
    
    sub_thread.join().unwrap();
    pub_thread.join().unwrap();
}

#[test]
fn pushpull() {
    let url = get_url();
    let factory = Latest::new();

    let pusher = factory.pusher_open().unwrap();
    pusher.listen(&url).unwrap();
    let puller = factory.puller_open().unwrap();
    puller.dial(&url).unwrap();
    let count = 4;
    let push_thread = thread::spawn(move || {
        let mut push_ctx = pusher.create_async_context().unwrap();
        for _ in 0..count {
            push_ctx.send(msg::NngMsg::new().unwrap()).wait().unwrap().unwrap();
        }
    });
    let recv_count = AtomicUsize::new(0);
    let pull_thread = thread::spawn(move || {
        let mut pull_ctx = puller.create_async_context().unwrap();
        for _ in 0..count {
            pull_ctx.receive().wait().unwrap().unwrap().unwrap();
            recv_count.fetch_add(1, Ordering::Relaxed);
        }
    });
    push_thread.join().unwrap();
    pull_thread.join().unwrap();
}

#[test]
fn broker() {
    let url_broker_in = get_url();
    let url_broker_out = get_url();

    let factory = Latest::new();

    let broker_pull = factory.puller_open().unwrap();
    let broker_push = factory.publisher_open().unwrap();
    broker_pull.listen(&url_broker_in).unwrap();
    broker_push.listen(&url_broker_out).unwrap();
    
    thread::sleep(Duration::from_millis(50));

    thread::spawn(move || {
        let mut broker_pull_ctx = broker_pull.create_async_context().unwrap();
        let mut broker_push_ctx = broker_push.create_async_context().unwrap();
        loop {
            let msg = broker_pull_ctx.receive().wait().unwrap().unwrap().unwrap();
            broker_push_ctx.send(msg).wait().unwrap().unwrap();
        }
    });

    let publisher = factory.pusher_open().unwrap();
    publisher.dial(&url_broker_in).unwrap();
    let subscriber = factory.subscriber_open().unwrap();
    subscriber.dial(&url_broker_out).unwrap();

    thread::spawn(move || {
        let mut sub_ctx = subscriber.create_async_context().unwrap();

        let topic: Vec<u8> = vec![0; 4];
        sub_ctx.subscribe(topic.as_slice()).unwrap();
        loop {
            let msg = sub_ctx.receive().wait().unwrap().unwrap().unwrap();
        }
    });

    thread::sleep(Duration::from_millis(50));

    thread::spawn(move || {
        let mut pub_ctx = publisher.create_async_context().unwrap();
        loop {
            let mut msg = msg::NngMsg::new().unwrap();
            msg.append_u32(0).unwrap();
            pub_ctx.send(msg).wait().unwrap().unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    });

    thread::sleep(Duration::from_secs(3));
}