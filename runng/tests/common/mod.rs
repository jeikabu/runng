use env_logger::{Builder, Env};
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn init_logging() {
    Builder::from_env(Env::default().default_filter_or("trace"))
        .try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));
}

static URL_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_url() -> String {
    init_logging();
    let val = URL_ID.fetch_add(1, Ordering::Relaxed);
    String::from("inproc://test") + &val.to_string()
}
