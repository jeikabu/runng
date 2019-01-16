use env_logger::{Builder, Env};
use futures::{future, Future, Stream};
use runng::{msg::NngMsg, NngResult};
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn init_logging() {
    Builder::from_env(Env::default().default_filter_or("debug"))
        .try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));
}

static URL_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_url() -> String {
    init_logging();
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}

pub fn create_stop_message() -> NngMsg {
    NngMsg::create().unwrap()
}

pub fn not_stop_message(res: &NngResult<NngMsg>) -> impl Future<Item = bool, Error = ()> {
    match res {
        Ok(msg) => future::ok(!msg.is_empty()),
        Err(_) => future::ok(false),
    }
}
