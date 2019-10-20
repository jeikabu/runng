use crate::common::*;
use log::debug;
use rand::Rng;
use runng::{
    asyncio::*,
    options::{GetOpts, NngOption},
};
use runng_sys::*;

fn create_listener_dialer_tcp() -> runng::Result<(StreamListener, StreamDialer)> {
    let url = "tcp://localhost:0";
    let listener = StreamListener::alloc(&url)?;
    listener.listen()?;
    let socket = listener.get_int(NngOption::TCP_BOUND_PORT)?;
    let url = format!("tcp://localhost:{}", socket);
    let dialer = StreamDialer::alloc(&url).unwrap();
    Ok((listener, dialer))
}

#[test]
fn stream() -> runng::Result<()> {
    init_logging();

    let (mut listener, mut dialer) = create_listener_dialer_tcp()?;
    let mut listener_aio = SimpleAioWorkQueue::new()?;
    let mut dialer_aio = SimpleAioWorkQueue::new()?;
    let accept_future = listener.accept(&mut listener_aio);
    let dial_future = dialer.dial(&mut dialer_aio);

    block_on(accept_future)??;
    block_on(dial_future)??;

    Ok(())
}

#[test]
fn iov() -> runng::Result<()> {
    init_logging();

    let (mut listener, mut dialer) = create_listener_dialer_tcp()?;
    let mut listener_aio = SimpleAioWorkQueue::new()?;
    let mut dialer_aio = SimpleAioWorkQueue::new()?;
    let accept_future = listener.accept(&mut listener_aio);
    let dial_future = dialer.dial(&mut dialer_aio);

    let mut listen_stream = block_on(accept_future)??;
    let mut dial_stream = block_on(dial_future)??;
    let mut original = vec![vec!(0u8, 128); 4];
    for iovx in original.iter_mut() {
        rand::thread_rng().fill(iovx.as_mut_slice());
    }
    let original = original;

    let fut = dial_stream.send(&mut dialer_aio, original.clone());
    block_on(fut)??;
    let iov = vec![vec!(0u8, 128); 4];
    let iov = block_on(listen_stream.recv(&mut listener_aio, iov))??;
    assert_eq!(iov, original);

    Ok(())
}
