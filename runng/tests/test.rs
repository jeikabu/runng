extern crate futures;
extern crate runng;
extern crate runng_sys;

use futures::future::Future;
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
    req_ctx.send().wait();
    //std::thread::sleep(Duration::from_millis(200));
}

#[test]
fn msg() {
    let mut builder = msg::MsgBuilder::new();
    let value: u32 = 0x01234567;
    builder.append_u32(value);
    let mut msg = builder.build().unwrap();
    assert_eq!(value, msg.trim_u32().unwrap());

    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut msg = builder.clean().append_slice(&data).build().unwrap();
    let mut nngmsg = msg::NngMsg::new().unwrap();
    nngmsg.append(data.as_ptr() as *const ::std::os::raw::c_void, data.len());
    assert_eq!(nngmsg.body(), msg.body());
}