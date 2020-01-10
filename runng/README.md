Rust high-level wrapper around [NNG](https://github.com/nanomsg/nng) (Nanomsg-Next-Gen):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

Features:  
- Use [nng_aio](https://nng.nanomsg.org/man/v1.2.2/nng_aio.5) for asynchronous I/O
- Use [nng_ctx](https://nng.nanomsg.org/man/v1.2.2/nng_ctx.5) for advanced protocol handling
- Leverage [futures](https://docs.rs/futures) crate for ease of use with [tokio](https://tokio.rs/) and eventual support of [`async`/`await`](https://github.com/rust-lang/rust/issues/50547)

## Examples

Simple:
```rust
use runng::{
    Dial, Listen, RecvMsg, SendMsg,
    factory::latest::ProtocolFactory, 
    msg::NngMsg,
    protocol::*,
};
fn simple_reqrep() -> Result<(), runng::Error> {
    const url: &str = "inproc://test";

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.sendmsg(NngMsg::create()?)?;
    rep.recv()?;

    Ok(())
}
```

Asynchronous I/O:
```rust
use futures::{
    future::Future,
    stream::Stream,
};
use runng::{
    Dial, Listen,
    asyncio::*,
    factory::latest::ProtocolFactory,
    msg::NngMsg,
    protocol::*,
};

fn async_reqrep() -> Result<(), runng::Error> {
    const url: &str = "inproc://test";

    let factory = ProtocolFactory::default();
    let mut rep_ctx = factory.replier_open()?.listen(&url)?.create_async()?;

    let mut req_ctx = factory.requester_open()?.dial(&url)?.create_async()?;
    let req_future = req_ctx.send(NngMsg::create()?);
    let _request = rep_ctx.receive().wait()?;
    rep_ctx.reply(NngMsg::create()?).wait()??;
    req_future.wait().unwrap()?;

    Ok(())
}
```

Additional examples [in `tests/` folder](https://github.com/jeikabu/runng/tree/master/runng/tests) and [runng_examples](https://github.com/jeikabu/runng_examples).
