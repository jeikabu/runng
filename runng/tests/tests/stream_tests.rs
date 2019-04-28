use crate::common::init_logging;
use futures::future::Future;
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

    accept_future.wait()??;
    dial_future.wait()??;

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

    let mut listen_stream = accept_future.wait()??;
    let mut dial_stream = dial_future.wait()??;
    let mut original = vec![vec!(0u8, 128); 4];
    for iovx in original.iter_mut() {
        rand::thread_rng().fill(iovx.as_mut_slice());
    }
    let original = original;

    dial_stream
        .send(&mut dialer_aio, original.clone())
        .wait()??;
    let iov = vec![vec!(0u8, 128); 4];
    let iov = listen_stream.recv(&mut listener_aio, iov).wait()??;
    assert_eq!(iov, original);

    Ok(())
}
