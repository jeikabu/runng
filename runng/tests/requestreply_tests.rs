extern crate env_logger;
extern crate futures;
extern crate runng;
extern crate runng_sys;

mod common;

#[cfg(test)]
mod tests {

use futures::{
    future::Future,
    Stream,
};
use runng::{
    *,
    protocol::*,
};
use common::get_url;


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

}