extern crate runng;
extern crate runng_sys;

use runng::*;
use runng_sys::nng_msg;
use runng::protocol::*;
use runng::socket::*;

#[test]
fn it_works() {
    let factory = Latest::new();
    let req = factory.requester_open().unwrap();
    let rep = factory.replier_open().unwrap();
    rep.listen("inproc://test").unwrap();
    req.dial("inproc://test").unwrap();
    req.send().unwrap();
    rep.recv().unwrap();
}

#[test]
fn aio() {
    let factory = Latest::new();
    let socket = factory.requester_open().unwrap();
    socket.dial("inproc://test").unwrap();
    let ctx = socket.create_async_context().unwrap();
    aio_ctx.send();
}