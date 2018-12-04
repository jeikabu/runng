//! Rust high-level wrapper around [NNG](https://github.com/nanomsg/nng) (Nanomsg-Next-Gen)
//! # Examples
//! Simple:
/*! ```
use runng::*;
fn test() -> Result<(), NngFail> {
    const url: &str = "inproc://test";
    let factory = Latest::new();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::new()?)?;
    rep.recv()?;
    Ok(())
}
```
*/
//! Asynchronous I/O:
/*! ```
extern crate futures;
extern crate runng;
use futures::{
    future::Future,
    stream::Stream,
};
use runng::{
    *,
    protocol::{
        AsyncReply,
        AsyncRequest,
        AsyncSocket,
    },
};

fn aio() -> NngReturn {
    const url: &str = "inproc://test";

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
```
*/

pub mod aio;
pub mod ctx;
pub mod dialer;
pub mod factory;
pub mod listener;
pub mod msg;
pub mod options;
pub mod pipe;
pub mod protocol;
pub mod result;
pub mod socket;
pub mod transport;

pub use self::aio::*;
pub use self::ctx::*;
pub use self::factory::*;
pub use self::options::*;
pub use self::result::*;
pub use self::socket::*;

extern crate futures;
extern crate runng_sys;
extern crate runng_derive;

#[macro_use]
extern crate log;

use runng_sys::*;

// Trait where type exposes a socket, but this shouldn't be part of public API
trait InternalSocket {
    fn socket(&self) -> &NngSocket;
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(string: &str) -> Result<(std::ffi::CString, *const i8), std::ffi::NulError> {
    let string = std::ffi::CString::new(string)?;
    let ptr = string.as_bytes_with_nul().as_ptr() as *const i8;
    Ok((string, ptr))
}