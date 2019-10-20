use crate::common::*;
use log::{debug, trace};
use runng::{
    asyncio::*,
    options::{NngOption, SetOpts},
    socket::*,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

fn wait_load(atomic: &Arc<AtomicUsize>, value: usize, done: &Arc<AtomicBool>) {
    while atomic.load(Ordering::SeqCst) < value && !done.load(Ordering::SeqCst) {
        sleep_fast();
    }
}

fn create_peer_sync(
    peer_urls: Vec<String>,
    peer_index: usize,
    done: Arc<AtomicBool>,
    peers_ready: Arc<AtomicUsize>,
    next_id: Arc<AtomicUsize>,
) -> thread::JoinHandle<runng::Result<()>> {
    thread::spawn(move || -> runng::Result<()> {
        let url = &peer_urls[peer_index];
        let mut socket = protocol::Bus0::open()?.listen(url)?;
        socket.socket_mut().set_ms(NngOption::RECVTIMEO, 100)?;

        let num_peers = peer_urls.len();
        debug!("Listening {} {}...", url, num_peers);
        let _ = peers_ready.fetch_add(1, Ordering::SeqCst);
        wait_load(&peers_ready, num_peers, &done);
        for (i, url_other) in peer_urls.iter().enumerate() {
            if i != peer_index {
                debug!("Dialing {} {}...", url, url_other);
                socket.dial_mut(&url_other)?;
            }
        }

        let _ = peers_ready.fetch_add(1, Ordering::SeqCst);
        wait_load(&peers_ready, num_peers * 2, &done);
        debug!("Ready {}", url);

        'outer: while !done.load(Ordering::SeqCst) {
            let id: u32 = next_id.fetch_add(1, Ordering::SeqCst) as u32;
            trace!("Fetch_add {} {}", url, id);
            // Wait for message containing previous id
            loop {
                if done.load(Ordering::SeqCst) {
                    break 'outer;
                }
                // If this is the first message, don't need to wait to receive one
                if id == 0 {
                    break;
                }
                match socket.recvmsg() {
                    Ok(mut msg) => {
                        let recv_id = msg.trim_u32()?;
                        trace!("Recv {} {} {}", url, id, recv_id);
                        // This is subtle.  If a node sends id X and loops around and gets id X+1
                        // it will end up waiting on itself (X).  However, a sender cannot receive
                        // their own messages.  Therefore we need to wait for either id X-1 or X-2.
                        if recv_id + 2 >= id {
                            break;
                        }
                    }
                    Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => debug!("Read timeout"),
                    err => panic!("Unexpected: {:?}", err),
                }
            }
            let mut msg = NngMsg::new()?;
            msg.append_u32(id)?;
            socket.sendmsg(msg)?;
            trace!("Sent {} {}", url, id);
        }

        Ok(())
    })
}

#[test]
fn bus_sync() -> runng::Result<()> {
    const NUM_PEERS: usize = 3;
    let peer_urls: Vec<_> = (0..NUM_PEERS).map(|_| get_url()).collect();

    let done = Arc::new(AtomicBool::default());
    let msg_id = Arc::new(AtomicUsize::new(0));
    let peers_ready = Arc::new(AtomicUsize::new(0));

    let peers: Vec<_> = (0..NUM_PEERS)
        .map(|i| {
            create_peer_sync(
                peer_urls.clone(),
                i,
                done.clone(),
                peers_ready.clone(),
                msg_id.clone(),
            )
        })
        .collect();
    sleep_test();
    done.store(true, Ordering::SeqCst);

    peers
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());
    // Assume every peer gets to send at least 2 messages
    assert!(msg_id.load(Ordering::SeqCst) > (NUM_PEERS + 1) * 2);

    Ok(())
}

fn create_peer_async(
    peer_urls: Vec<String>,
    peer_index: usize,
    done: Arc<AtomicBool>,
    peers_ready: Arc<AtomicUsize>,
    next_id: Arc<AtomicUsize>,
) -> thread::JoinHandle<runng::Result<()>> {
    thread::spawn(move || -> runng::Result<()> {
        let url = &peer_urls[peer_index];
        let mut socket = protocol::Bus0::open()?.listen(url)?;
        socket.socket_mut().set_ms(NngOption::RECVTIMEO, 100)?;
        let mut ctx = socket.create_async()?;

        let num_peers = peer_urls.len();
        let _ = peers_ready.fetch_add(1, Ordering::Relaxed);
        wait_load(&peers_ready, num_peers, &done);
        for (i, url_other) in peer_urls.iter().enumerate() {
            if i != peer_index {
                debug!("Dialing {} {}...", url, url_other);
                socket.dial_mut(&url_other)?;
            }
        }

        let _ = peers_ready.fetch_add(1, Ordering::SeqCst);
        wait_load(&peers_ready, num_peers * 2, &done);
        debug!("Ready {}", url);

        'outer: while !done.load(Ordering::Relaxed) {
            let id: u32 = next_id.fetch_add(1, Ordering::Relaxed) as u32;
            // Wait for message containing previous id
            loop {
                if done.load(Ordering::Relaxed) {
                    break 'outer;
                }
                // If this is the first message, don't need to wait to receive one
                if id == 0 {
                    break;
                }
                match block_on(ctx.receive()) {
                    Ok(mut msg) => {
                        let recv_id = msg.trim_u32()?;
                        trace!("Recv {} {}", id, recv_id);
                        if recv_id + 2 >= id {
                            break;
                        }
                    }
                    Err(runng::Error::Errno(NngErrno::ETIMEDOUT)) => debug!("Read timeout"),
                    err => panic!("Unexpected: {:?}", err),
                }
            }
            let mut msg = NngMsg::new()?;
            msg.append_u32(id)?;
            block_on(ctx.send(msg))?;
            trace!("Sent {} {}", url, id);
        }

        Ok(())
    })
}

#[test]
fn bus() -> runng::Result<()> {
    const NUM_PEERS: usize = 2;
    let peer_urls: Vec<_> = (0..NUM_PEERS).map(|_| get_url()).collect();

    let done = Arc::new(AtomicBool::default());
    let msg_id = Arc::new(AtomicUsize::new(0));
    let peers_ready = Arc::new(AtomicUsize::new(0));

    let peers: Vec<_> = (0..NUM_PEERS)
        .map(|i| {
            create_peer_async(
                peer_urls.clone(),
                i,
                done.clone(),
                peers_ready.clone(),
                msg_id.clone(),
            )
        })
        .collect();
    sleep_test();
    done.store(true, Ordering::Relaxed);

    peers
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());
    // Assume every peer gets to send at least 2 messages
    assert!(msg_id.load(Ordering::Relaxed) > (NUM_PEERS + 1) * 2);

    Ok(())
}
