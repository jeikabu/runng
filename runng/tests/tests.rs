mod common;

#[cfg(test)]
mod tests {

    mod broker_tests;
    mod future_tests;
    mod mem_tests;
    mod msg_tests;
    mod options_tests;
    mod pair_tests;
    mod pipe_tests;
    mod pushpull_tests;
    mod reqrep_tests;
    mod stats_tests;
    mod stream_tests;

    use crate::common::get_url;
    use futures::{future, future::Future, Stream};
    use runng::{asyncio::*, factory::latest::ProtocolFactory, protocol::*, *};
    use std::{thread, time::Duration};

    #[test]
    fn listenerdialer() -> runng::Result<()> {
        let url = get_url();
        let factory = ProtocolFactory::default();

        let replier = factory.replier_open()?;
        {
            {
                let listener = replier.listener_create(&url)?;
                listener.start()?;
                let requester = factory.requester_open()?;
                {
                    let req_dialer = requester.dialer_create(&url)?;
                    assert_eq!(
                        url,
                        req_dialer
                            .get_string(NngOption::URL)
                            .unwrap()
                            .to_str()
                            .unwrap()
                    );
                    req_dialer.start()?;
                    requester.sendmsg(msg::NngMsg::new()?)?;
                    let _request = replier.recvmsg()?;
                    // Drop the dialer
                }
                // requester still works
                requester.sendmsg(msg::NngMsg::new()?)?;
                let _request = replier.recvmsg()?;
                // Drop the listener
            }
            // Replier still works
            let requester = factory.requester_open()?.dial(&url)?;
            requester.sendmsg(msg::NngMsg::new()?)?;
            let _request = replier.recvmsg()?;
        }

        Ok(())
    }

    #[test]
    fn pubsub() -> runng::Result<()> {
        let url = get_url();
        let factory = ProtocolFactory::default();

        let publisher = factory.publisher_open()?.listen(&url)?;
        let subscriber = factory.subscriber_open()?.dial(&url)?;

        let num_msg_per_subscriber = 4;

        let sub_thread = thread::spawn(move || -> runng::Result<()> {
            let mut sub_ctx = subscriber.create_async_stream(1)?;
            let topic: Vec<u8> = vec![0; 4];
            sub_ctx.subscribe(topic.as_slice())?;

            sub_ctx
                .receive()
                .unwrap()
                // Process until receive stop message
                .take_while(|res| {
                    const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                    match res {
                        Ok(msg) => future::ok(msg.len() - SIZE_OF_TOPIC > 0),
                        Err(_) => future::ok(false),
                    }
                })
                // Increment count of received messages
                .for_each(|_| {
                    //thread_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .wait()
                .unwrap();
            Ok(())
        });
        let pub_thread = thread::spawn(move || -> runng::Result<()> {
            let mut pub_ctx = publisher.create_async()?;

            // Beginning of message body contains topic
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?; // topic
            msg.append_u32(1)?;

            for _ in 0..num_msg_per_subscriber {
                let msg = msg.dup()?;
                pub_ctx.send(msg).wait().unwrap()?;
                thread::sleep(Duration::from_millis(25));
            }

            // Send stop message
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?; // topic
            pub_ctx.send(msg).wait().unwrap()?;
            Ok(())
        });

        sub_thread.join().unwrap()?;
        pub_thread.join().unwrap()?;

        Ok(())
    }

    #[test]
    fn broker() -> runng::Result<()> {
        let url_broker_in = get_url();
        let url_broker_out = get_url();

        let factory = ProtocolFactory::default();

        let broker_pull = factory.puller_open()?.listen(&url_broker_in)?;
        let broker_push = factory.publisher_open()?.listen(&url_broker_out)?;

        thread::sleep(Duration::from_millis(50));

        // Broker
        thread::spawn(move || -> runng::Result<()> {
            let mut broker_pull_ctx = broker_pull.create_async_stream(1)?;
            let mut broker_push_ctx = broker_push.create_async()?;
            broker_pull_ctx
                .receive()
                .unwrap()
                .for_each(|msg| {
                    if let Ok(msg) = msg {
                        broker_push_ctx.send(msg).wait().unwrap().unwrap();
                    }
                    Ok(())
                })
                .wait()
                .unwrap();

            Ok(())
        });

        let publisher = factory.pusher_open()?.dial(&url_broker_in)?;
        let subscriber = factory.subscriber_open()?.dial(&url_broker_out)?;

        // Subscriber
        thread::spawn(move || -> runng::Result<()> {
            let mut sub_ctx = subscriber.create_async_stream(1)?;

            let topic: Vec<u8> = vec![0; 4];
            sub_ctx.subscribe(topic.as_slice())?;
            sub_ctx
                .receive()
                .unwrap()
                // Process until receive stop message
                .take_while(|res| {
                    const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                    match res {
                        Ok(msg) => future::ok(msg.len() - SIZE_OF_TOPIC > 0),
                        Err(_) => future::ok(false),
                    }
                })
                .for_each(|_| {
                    //thread_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .wait()
                .unwrap();
            Ok(())
        });

        thread::sleep(Duration::from_millis(50));

        // Publisher
        thread::spawn(move || -> runng::Result<()> {
            let mut pub_ctx = publisher.create_async()?;
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
