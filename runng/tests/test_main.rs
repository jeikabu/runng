mod common;

#[cfg(test)]
mod tests {

    mod broker_tests;
    mod bus_tests;
    mod future_tests;
    mod mem_tests;
    mod msg_tests;
    mod options_tests;
    mod pair_tests;
    mod pipe_tests;
    mod pubsub_tests;
    mod pushpull_tests;
    mod reqrep_tests;
    mod stats_tests;
    mod stream_tests;

    use crate::common::*;
    use futures::{executor::block_on, future};
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
            let mut requester = factory.requester_open()?;
            requester.dial(&url)?;
            requester.sendmsg(msg::NngMsg::new()?)?;
            let _request = replier.recvmsg()?;
        }

        Ok(())
    }

    #[test]
    fn results() -> std::result::Result<(), runng_sys::EnumFromIntError> {
        use core::convert::TryFrom;
        assert_eq!(
            NngErrno::EINTR,
            NngErrno::try_from(runng_sys::NNG_EINTR as i32).unwrap()
        );
        Ok(())
    }

    #[test]
    fn pubsub() -> runng::Result<()> {
        let url = get_url();
        let factory = ProtocolFactory::default();

        let mut publisher = factory.publisher_open()?;
        publisher.listen(&url)?;
        let mut subscriber = factory.subscriber_open()?;
        subscriber.dial(&url)?;

        let num_msg_per_subscriber = 4;

        let sub_thread = thread::spawn(move || -> runng::Result<()> {
            let mut sub_ctx = subscriber.create_async_stream(1)?;
            let topic: Vec<u8> = vec![0; 4];
            sub_ctx.subscribe(topic.as_slice())?;

            let fut = sub_ctx
                .receive()
                .unwrap()
                // Process until receive stop message
                .take_while(|res| {
                    const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                    let is_taking = match res {
                        Ok(msg) => msg.len() - SIZE_OF_TOPIC > 0,
                        Err(_) => false,
                    };
                    future::ready(is_taking)
                })
                // Increment count of received messages
                .for_each(|_| {
                    //thread_count.fetch_add(1, Ordering::Relaxed);
                    future::ready(())
                });
            block_on(fut);
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
                block_on(pub_ctx.send(msg))?;
                thread::sleep(Duration::from_millis(25));
            }

            // Send stop message
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?; // topic
            block_on(pub_ctx.send(msg))?;
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

        let mut broker_pull = factory.puller_open()?;
        broker_pull.listen(&url_broker_in)?;
        let mut broker_push = factory.publisher_open()?;
        broker_push.listen(&url_broker_out)?;

        thread::sleep(Duration::from_millis(50));

        // Broker
        thread::spawn(move || -> runng::Result<()> {
            let mut broker_pull_ctx = broker_pull.create_async_stream(1)?;
            let mut broker_push_ctx = broker_push.create_async()?;
            let fut = broker_pull_ctx.receive().unwrap().for_each(|msg| {
                if let Ok(msg) = msg {
                    futures::future::Either::Left(broker_push_ctx.send(msg).then(|res| {
                        res.unwrap();
                        future::ready(())
                    }))
                } else {
                    futures::future::Either::Right(future::ready(()))
                }
            });
            block_on(fut);
            Ok(())
        });

        let mut publisher = factory.pusher_open()?;
        publisher.dial(&url_broker_in)?;
        let mut subscriber = factory.subscriber_open()?;
        subscriber.dial(&url_broker_out)?;

        // Subscriber
        thread::spawn(move || -> runng::Result<()> {
            let mut sub_ctx = subscriber.create_async_stream(1)?;

            let topic: Vec<u8> = vec![0; 4];
            sub_ctx.subscribe(topic.as_slice())?;
            let fut = sub_ctx
                .receive()
                .unwrap()
                // Process until receive stop message
                .take_while(|res| {
                    const SIZE_OF_TOPIC: usize = std::mem::size_of::<u32>();
                    let is_taking = match res {
                        Ok(msg) => msg.len() - SIZE_OF_TOPIC > 0,
                        Err(_) => false,
                    };
                    future::ready(is_taking)
                })
                .for_each(|_| {
                    //thread_count.fetch_add(1, Ordering::Relaxed);
                    future::ready(())
                });
            block_on(fut);
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
                block_on(pub_ctx.send(msg))?;
                thread::sleep(Duration::from_millis(200));
            }
            // Send stop message
            let mut msg = msg::NngMsg::new()?;
            msg.append_u32(0)?; // topic
            block_on(pub_ctx.send(msg))?;

            Ok(())
        });

        thread::sleep(Duration::from_secs(3));

        Ok(())
    }
}
