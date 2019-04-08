/*!
Rust high-level wrapper around [NNG](https://github.com/nanomsg/nng) (Nanomsg-Next-Gen):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

Features:
- Use [nng_aio](https://nanomsg.github.io/nng/man/v1.1.0/nng_aio.5) for asynchronous I/O
- Use [nng_ctx](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx.5) for advanced protocol handling
- Leverage [futures](https://docs.rs/futures) crate for ease of use with [tokio](https://tokio.rs/) and eventual support of [`async`/`await`](https://github.com/rust-lang/rust/issues/50547)

## Examples

Simple:
```rust
use runng::{
    Dial, Listen, RecvSocket, SendSocket,
    factory::latest::ProtocolFactory,
    msg::NngMsg,
    protocol::*,
};
fn simple_reqrep() -> Result<(), runng::Error> {
    const url: &str = "inproc://test";

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.sendmsg(NngMsg::new()?)?;
    rep.recvmsg()?;

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
    let req_future = req_ctx.send(NngMsg::new()?);
    let _request = rep_ctx.receive().wait()?;
    rep_ctx.reply(NngMsg::new()?).wait()??;
    req_future.wait().unwrap()?;

    Ok(())
}
```

Additional examples [in `examples/` folder](https://github.com/jeikabu/runng/tree/master/runng/examples).

*/

pub mod asyncio;
pub mod ctx;
pub mod dialer;
pub mod factory;
pub mod listener;
pub mod mem;
pub mod msg;
pub mod options;
pub mod pipe;
pub mod protocol;
pub mod result;
pub mod socket;
pub mod stats;
pub mod transport;

pub use self::ctx::*;
pub use self::factory::*;
pub use self::mem::NngString;
pub use self::options::*;
pub use self::result::*;
pub use self::socket::*;

use log::{debug, trace};
use runng_sys::*;

// Trait where type exposes a socket, but this shouldn't be part of public API
trait InternalSocket {
    fn socket(&self) -> &NngSocket;
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}

// Return string and pointer so string isn't dropped
fn to_cstr(
    string: &str,
) -> std::result::Result<(std::ffi::CString, *const std::os::raw::c_char), std::ffi::NulError> {
    let string = std::ffi::CString::new(string)?;
    let ptr = string.as_bytes_with_nul().as_ptr() as *const std::os::raw::c_char;
    Ok((string, ptr))
}
