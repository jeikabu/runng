use env_logger::{Builder, Env};
pub use futures::{
    executor::block_on,
    future::{self, Either, Future},
};
pub use futures_util::{future::FutureExt, stream::StreamExt};
pub use log::{debug, info, trace};
use rand::Rng;
pub use runng::{asyncio, msg::NngMsg, protocol, NngErrno};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread, time,
    time::Duration,
};

pub fn init_logging() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));
}

static URL_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_url() -> String {
    init_logging();
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}

pub fn get_urls() -> impl Iterator<Item = String> {
    init_logging();
    vec![
        "ws://localhost:0".to_owned(),
        "tcp://localhost:0".to_owned(),
    ]
    .into_iter()
}

pub fn create_stop_message() -> NngMsg {
    NngMsg::new().unwrap()
}

pub fn not_stop_message(res: &runng::Result<NngMsg>) -> impl Future<Output = bool> {
    future::ready(if let Ok(msg) = res {
        !msg.is_empty()
    } else {
        false
    })
}

pub const DURATION_FAST: time::Duration = time::Duration::from_millis(10);
pub const DURATION_BRIEF: time::Duration = time::Duration::from_millis(25);
pub const DURATION_LONG: time::Duration = time::Duration::from_millis(75);
pub const DURATION_TEST: time::Duration = time::Duration::from_secs(1);

pub fn sleep_fast() {
    thread::sleep(DURATION_FAST);
}

pub fn sleep_brief() {
    thread::sleep(DURATION_BRIEF);
}

pub fn sleep_test() {
    thread::sleep(DURATION_TEST);
}

pub fn rand_msg() -> runng::Result<NngMsg> {
    let mut msg = NngMsg::with_capacity(128)?;
    rand::thread_rng().fill(msg.as_mut_slice());
    Ok(msg)
}

pub fn rand_sleep(low: u64, high: u64) {
    let range = rand::thread_rng().gen_range(low, high);
    thread::sleep(Duration::from_millis(range));
}

pub enum TimeoutResult<F: Future> {
    Ok(F::Output),
    Timeout(F),
}

pub fn timeout<F: Future + std::marker::Unpin>(
    future: F,
    duration: std::time::Duration,
) -> impl Future<Output = TimeoutResult<F>> {
    let timeout = futures_timer::Delay::new(duration);
    future::select(future, timeout).then(|either| match either {
        Either::Left((item, _timeout_future)) => future::ready(TimeoutResult::Ok(item)),
        Either::Right((_timeout_error, future)) => future::ready(TimeoutResult::Timeout(future)),
    })
}
