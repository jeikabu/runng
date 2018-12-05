//! Rust nngcat.  See [nngcat](https://nanomsg.github.io/nng/man/v1.1.0/nngcat.1).
//! `cargo run --example runngcat`

extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate runng;
extern crate tokio;

use env_logger::{Builder, Env};
use futures::{
    Future,
    future::lazy,
    Stream,
};
use runng::{
    *,
    protocol::AsyncContext,
    protocol::AsyncSocket,
    protocol::AsyncReply,
    protocol::AsyncReplyContext,
    protocol::Subscribe,
};

fn main() -> NngReturn {
    Builder::from_env(Env::default().default_filter_or("debug")).try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));

    tokio::run(lazy(|| {
        let mut replier = create_echo().unwrap();
        replier.receive()
            .for_each(move |msg|{
                if let Ok(msg) = msg {
                    info!("Echo {:?}", msg);
                    replier.reply(msg).wait().unwrap();
                }
                
                Ok(())
            })
    }));
    Ok(())
}

fn create_echo() -> NngResult<Box<AsyncReplyContext>> {
    let url = "tcp://127.0.0.1:9823";
    let factory = Latest::default();
    let replier = factory.replier_open()?
        .listen(&url)?
        .create_async_context()?;
    Ok(replier)
}