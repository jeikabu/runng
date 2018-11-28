# runng

High-level wrapper around [NNG](https://github.com/nanomsg/nng).

Features:  
- Use of [nng_aio](https://nanomsg.github.io/nng/man/v1.1.0/nng_aio.5) for asynchronous I/O
- Use of [nng_ctx](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx.5) for advanced protocol handling
- Leverage [futures](https://docs.rs/futures) crate for ease of use with [tokio](https://tokio.rs/) and eventual support of [`async`/`await`](https://github.com/rust-lang/rust/issues/50547)

## Sample

```rust
#[test]
fn aio() -> Result<(), NngFail> {
    let url = String::from("inproc://test");

    let factory = Latest::new();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;

    let requester = factory.requester_open()?.dial(&url)?;
    let mut req_ctx = requester.create_async_context()?;
    let req_future = req_ctx.send(msg::NngMsg::new()?);
    rep_ctx.receive()
        .take(1).for_each(|request|{
            let msg = msg::NngMsg::new().unwrap();
            rep_ctx.reply(msg).wait().unwrap();
            Ok(())
        }).wait();
    req_future.wait().unwrap()?;

    Ok(())
}
```