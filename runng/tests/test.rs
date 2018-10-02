extern crate runng;

use runng::*;

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