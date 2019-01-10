use crate::common::get_url;
use futures::{future::Future, Stream};
use log::info;
use runng::{protocol::*, *};

#[test]
fn example_basic() -> NngReturn {
    info!("basic");
    let url = get_url();

    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::create()?)?;
    rep.recv()?;

    Ok(())
}

#[test]
fn example_async() -> NngReturn {
    info!("async");
    let url = get_url();

    let factory = Latest::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;

    let requester = factory.requester_open()?.dial(&url)?;
    let mut req_ctx = requester.create_async_context()?;
    let req_future = req_ctx.send(msg::NngMsg::create()?);
    rep_ctx
        .receive()
        .unwrap()
        .take(1)
        .for_each(|_request| {
            let msg = msg::NngMsg::create().unwrap();
            rep_ctx.reply(msg).wait().unwrap().unwrap();
            Ok(())
        })
        .wait()?;
    req_future.wait().unwrap()?;

    Ok(())
}
