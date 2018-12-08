/*!
Tokio running reply socket that accepts requests and responds with the same data.
## Examples
```
# Defaults to listening at tcp://127.0.0.1:9823
cargo run --example tokio_echo
cargo run --example runngcat -- --req0 --dial tcp://127.0.0.1:9823 --data hi
```
*/ 

extern crate clap;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate runng;
extern crate tokio;

use clap::{Arg, ArgMatches, App};
use env_logger::{Builder, Env};
use futures::{
    Future,
    future::lazy,
    Stream,
};
use runng::{
    *,
    protocol::*,
};

fn main() -> NngReturn {
    Builder::from_env(Env::default().default_filter_or("debug")).try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));

    let matches = get_matches();

    tokio::run(lazy(move || {
        let mut replier = create_echo(&matches).unwrap();
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

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("tokio_echo")
        .arg(Arg::with_name("listen")
            .long("listen")
            .help("Bind to and accept connections at specified address")
            .default_value("tcp://127.0.0.1:9823")
        )
        .get_matches()
}

fn create_echo<'a>(matches: &ArgMatches<'a>) -> NngResult<Box<AsyncReplyContext>> {
    let url = matches.value_of("listen").unwrap();
    let factory = Latest::default();
    let replier = factory.replier_open()?
        .listen(&url)?
        .create_async_context()?;
    Ok(replier)
}