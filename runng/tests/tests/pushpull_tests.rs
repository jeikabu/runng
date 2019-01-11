use crate::common::get_url;
use futures::{future, future::Future, Stream};
use runng::{protocol::*, *};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

#[test]
fn pushpull() -> NngReturn {
    let url = get_url();
    let factory = Latest::default();

    let pusher = factory.pusher_open()?.listen(&url)?;
    let puller = factory.puller_open()?.dial(&url)?;
    let count = 4;

    // Pusher
    let push_thread = thread::spawn(move || -> NngReturn {
        let mut push_ctx = pusher.create_async_context()?;
        // Send messages
        for i in 0..count {
            let msg = msg::MsgBuilder::default().append_u32(i).build()?;
            push_ctx.send(msg).wait().unwrap()?;
        }
        // Send a stop message
        let stop_message = msg::NngMsg::create().unwrap();
        push_ctx.send(stop_message).wait().unwrap()?;
        Ok(())
    });

    // Puller
    let recv_count = Arc::new(AtomicUsize::new(0));
    let thread_count = recv_count.clone();
    let pull_thread = thread::spawn(move || -> NngReturn {
        let mut pull_ctx = puller.create_async_context()?;
        pull_ctx
            .receive()
            .unwrap()
            // Process until receive stop message
            .take_while(|res| match res {
                Ok(msg) => future::ok(!msg.is_empty()),
                Err(_) => future::ok(false),
            })
            // Increment count of received messages
            .for_each(|_| {
                thread_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .wait()
            .unwrap();
        Ok(())
    });

    push_thread.join().unwrap()?;
    pull_thread.join().unwrap()?;

    // Received number of messages we sent
    assert_eq!(recv_count.load(Ordering::Relaxed), count as usize);

    Ok(())
}

// #[test]
// fn crash() -> NngReturn {
//     let url = get_url();
//     let factory = Latest::default();

//     let pusher = factory.pusher_open()?.listen(&url)?;
//     let puller = factory.puller_open()?.dial(&url)?;

//     // Pusher
//     let push_thread = thread::spawn(move || -> NngReturn {
//         let mut push_ctx = pusher.create_async_context()?;
//         // Send messages
//         loop {
//             let msg = msg::NngMsg::create()?;
//             push_ctx.send(msg).wait().unwrap()?;
//         }
//         Ok(())
//     });

//     // Puller
//     let recv_count = Arc::new(AtomicUsize::new(0));
//     let thread_count = recv_count.clone();
//     let pull_thread = thread::spawn(move || -> NngReturn {
//         puller.create_async_context()?
//             .receive().unwrap()
//             .for_each(|msg| {
//                 debug!("Pulled: {:?}", msg);
//                 Ok(())
//             }
//             )
//             .wait().unwrap();
//         Ok(())
//     });

//     push_thread.join().unwrap();
//     pull_thread.join().unwrap();

//     Ok(())
// }
