extern crate env_logger;
extern crate futures;
extern crate runng;
extern crate runng_sys;

#[cfg(test)]
mod tests {

use env_logger::{Builder, Env};

use futures::{
    future,
    future::{
        Future,
    },
    Stream,
};
use runng::protocol::*;
use runng::socket::*;
use runng::*;
use std::{
    sync::Arc,
    thread,
    time::Duration
};

use std::sync::atomic::{AtomicUsize, Ordering};

static URL_ID: AtomicUsize = AtomicUsize::new(1);
fn get_url() -> String {
    Builder::from_env(Env::default().default_filter_or("trace")).try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}

#[test]
fn it_works() -> NngReturn {
    let url = get_url();

    let factory = Latest::new();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::new()?)?;
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
    rep_ctx.receive()
        .take(1).for_each(|_request|{
            let msg = msg::NngMsg::new().unwrap();
            rep_ctx.reply(msg).wait().unwrap();
            Ok(())
        }).wait();
    req_future.wait().unwrap()?;

    Ok(())
}

#[test]
fn msg() -> NngReturn {
    let mut builder = msg::MsgBuilder::default();
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
fn listenerdialer() -> NngReturn {
    let url = get_url();
    let factory = Latest::new();

    let replier = factory.replier_open()?;
    {
        {
            let listener = replier.listener_create(&url)?;
            listener.start()?;
            let requester = factory.requester_open()?;
            {
                let req_dialer = requester.dialer_create(&url)?;
                assert_eq!(url, req_dialer.getopt_string(NngOption::URL).unwrap().to_str().unwrap());
                req_dialer.start()?;
                requester.send(msg::NngMsg::new()?)?;
                let _request = replier.recv()?;
                // Drop the dialer
            }
            // requester still works
            requester.send(msg::NngMsg::new()?)?;
            let _request = replier.recv()?;
            // Drop the listener
        }
        // Replier still works
        let requester = factory.requester_open()?.dial(&url)?;
        requester.send(msg::NngMsg::new()?)?;
        let _request = replier.recv()?;
    }

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

        sub_ctx.receive()
            // Process until receive stop message
            .take_while(|res| {
                const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                match res {
                    Ok(msg) => future::ok(msg.len() - SIZE_OF_TOPIC > 0),
                    Err(_) => future::ok(false),
                }
            })
            // Increment count of received messages
            .for_each(|_|{
                //thread_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }).wait().unwrap();
        Ok(())
    });
    let pub_thread = thread::spawn(move || -> NngReturn {
        let mut pub_ctx = publisher.create_async_context()?;

        // Beginning of message body contains topic
        let msg = msg::MsgBuilder::default()
            .append_u32(0) // topic
            .append_u32(1)
            .build()?;

        for _ in 0..num_msg_per_subscriber {
            let msg = msg.dup()?;
            pub_ctx.send(msg).wait().unwrap()?;
            thread::sleep(Duration::from_millis(25));
        }

        // Send stop message
        let msg = msg::MsgBuilder::default()
            .append_u32(0) // topic
            .build()?;
        pub_ctx.send(msg).wait().unwrap()?;
        Ok(())
    });

    sub_thread.join().unwrap()?;
    pub_thread.join().unwrap()?;

    Ok(())
}

#[test]
fn pushpull() -> NngReturn {
    let url = get_url();
    let factory = Latest::new();

    let pusher = factory.pusher_open()?.listen(&url)?;
    let puller = factory.puller_open()?.dial(&url)?;
    let count = 4;

    // Pusher
    let push_thread = thread::spawn(move || -> NngReturn {
        let mut push_ctx = pusher.create_async_context()?;
        // Send messages
        for i in 0..count {
            let msg = msg::MsgBuilder::default()
                .append_u32(i).build()?;
            push_ctx
                .send(msg)
                .wait()
                .unwrap()?;
        }
        // Send a stop message
        let stop_message = msg::NngMsg::new().unwrap();
        push_ctx
            .send(stop_message)
            .wait()
            .unwrap()?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let thread_count = recv_count.clone();
    let pull_thread = thread::spawn(move || -> NngReturn {
        let mut pull_ctx = puller.create_async_context()?;
        pull_ctx.receive()
            // Process until receive stop message
            .take_while(|res| {
                match res {
                    Ok(msg) => future::ok(msg.len() > 0),
                    Err(_) => future::ok(false),
                }
            })
            // Increment count of received messages
            .for_each(|_|{
                thread_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }).wait().unwrap();
        Ok(())
    });

    push_thread.join().unwrap();
    pull_thread.join().unwrap();

    // Received number of messages we sent
    assert_eq!(recv_count.load(Ordering::Relaxed), count as usize);

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

    // Broker
    thread::spawn(move || -> NngReturn {
        let mut broker_pull_ctx = broker_pull.create_async_context()?;
        let mut broker_push_ctx = broker_push.create_async_context()?;
        broker_pull_ctx.receive().for_each(|msg|{
            if let Ok(msg) = msg {
                broker_push_ctx.send(msg).wait().unwrap();
            }
            Ok(())
        }).wait().unwrap();
        
        Ok(())
    });

    let publisher = factory.pusher_open()?.dial(&url_broker_in)?;
    let subscriber = factory
        .subscriber_open()?
        .dial(&url_broker_out)?;

    // Subscriber
    thread::spawn(move || -> NngReturn {
        let mut sub_ctx = subscriber.create_async_context()?;

        let topic: Vec<u8> = vec![0; 4];
        sub_ctx.subscribe(topic.as_slice())?;
        sub_ctx.receive()
            // Process until receive stop message
            .take_while(|res| {
                const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                match res {
                    Ok(msg) => future::ok(msg.len() - SIZE_OF_TOPIC > 0),
                    Err(_) => future::ok(false),
                }
            })
            .for_each(|_|{
                //thread_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }).wait().unwrap();
        Ok(())
    });

    thread::sleep(Duration::from_millis(50));

    // Publisher
    thread::spawn(move || -> NngReturn {
        let mut pub_ctx = publisher.create_async_context()?;
        for _ in 0..10 {
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?; // topic
            msg.append_u32(1)?;
            pub_ctx.send(msg).wait().unwrap()?;
            thread::sleep(Duration::from_millis(200));
        }
        // Send stop message
        let mut msg = msg::NngMsg::new()?;
        msg.append_u32(0)?; // topic
        pub_ctx.send(msg).wait().unwrap()?;

        Ok(())
    });

    thread::sleep(Duration::from_secs(3));

    Ok(())
}

}