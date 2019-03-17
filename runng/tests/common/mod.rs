use env_logger::{Builder, Env};
use futures::{
    future,
    future::{Either, Future},
};
use rand::Rng;
use runng::msg::NngMsg;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread, time,
};

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

pub fn not_stop_message(res: &runng::Result<NngMsg>) -> impl Future<Item = bool, Error = ()> {
    match res {
        Ok(msg) => future::ok(!msg.is_empty()),
        Err(_) => future::ok(false),
    }
}

pub fn sleep_fast() {
    thread::sleep(time::Duration::from_millis(10));
}

pub fn sleep_brief() {
    thread::sleep(time::Duration::from_millis(25));
}

pub fn sleep_test() {
    thread::sleep(time::Duration::from_secs(1));
}

pub fn rand_msg() -> runng::Result<NngMsg> {
    let mut msg = NngMsg::with_size(128)?;
    rand::thread_rng().fill(msg.as_mut_slice());
    Ok(msg)
}

pub enum TimeoutResult<F: Future> {
    Ok(F::Item),
    Timeout(F),
}

pub fn timeout<F: Future>(
    future: F,
    duration: std::time::Duration,
) -> impl Future<Item = TimeoutResult<F>, Error = ()> {
    let timeout = futures_timer::Delay::new(duration);
    future.select2(timeout).then(|res| match res {
        Ok(Either::A((item, _timeout_future))) => future::ok(TimeoutResult::Ok(item)),
        Ok(Either::B((_timeout_error, future))) => future::ok(TimeoutResult::Timeout(future)),
        _ => future::err(()),
    })
}
