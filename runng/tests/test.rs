extern crate runng;

use runng::*;

#[test]
fn it_works() {
    let factory = Latest::new();
    let req = factory.requester_open().unwrap();
    let rep = factory.replier_open().unwrap();
}