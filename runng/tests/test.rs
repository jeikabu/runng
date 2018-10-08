extern crate runng;
extern crate runng_sys;

use runng::*;
use runng_sys::nng_msg;
use runng::protocol::*;
use runng::socket::*;
use std::{
    thread,
    time::Duration
};

#[test]
fn it_works() {
    let url = "inproc://test";

    let factory = Latest::new();
    let req = factory.requester_open().unwrap();
    let rep = factory.replier_open().unwrap();
    rep.listen(url).unwrap();
    req.dial(url).unwrap();
    req.send().unwrap();
    rep.recv().unwrap();
}

#[test]
fn aio() {
    let url = "inproc://test2";

    let factory = Latest::new();
    let replier = factory.replier_open().unwrap();
    replier.listen(url).unwrap();

    let requester = factory.requester_open().unwrap();
    requester.dial(url).unwrap();
    let mut req_ctx = requester.create_async_context().unwrap();
    req_ctx.send();
    std::thread::sleep(Duration::from_secs(1));
}