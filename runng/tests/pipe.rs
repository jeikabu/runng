#![cfg(feature = "pipes")]

extern crate env_logger;
extern crate futures;
extern crate runng;
extern crate runng_sys;

mod common;

#[cfg(test)]
mod tests {

use common::get_url;
use runng::*;
use runng::pipe::*;
use runng_sys::nng_pipe;
use std::sync::atomic::{AtomicUsize, Ordering};

static NumAddPre: AtomicUsize = AtomicUsize::new(0);
static NumAddPost: AtomicUsize = AtomicUsize::new(0);
static NumRemPost: AtomicUsize = AtomicUsize::new(0);
static NumBad: AtomicUsize = AtomicUsize::new(0);

extern fn notify_callback(pipe: nng_pipe, event: i32, arg1: PipeNotifyCallbackArg) {
    match PipeEvent::from_i32(event) {
        Some(PipeEvent::AddPre) => NumAddPre.fetch_add(1, Ordering::Relaxed),
        Some(PipeEvent::AddPost) => NumAddPost.fetch_add(1, Ordering::Relaxed),
        Some(PipeEvent::RemPost) => NumRemPost.fetch_add(1, Ordering::Relaxed),
        _ => NumBad.fetch_add(1, Ordering::Relaxed),
    };
}

#[test]
fn notify() -> NngReturn {
    let url = get_url();

    let factory = Latest::new();
    let rep = factory.replier_open()?.listen(&url)?;
    [PipeEvent::AddPre, PipeEvent::AddPost, PipeEvent::RemPost].iter().
        for_each(|event| rep.socket().notify(*event, notify_callback, std::ptr::null_mut()).unwrap());
    {
        let _ = factory.requester_open()?.dial(&url)?;
    }
    
    assert_eq!(NumAddPre.load(Ordering::Relaxed), 1);
    assert_eq!(NumAddPost.load(Ordering::Relaxed), 1);
    assert_eq!(NumRemPost.load(Ordering::Relaxed), 1);
    assert_eq!(NumBad.load(Ordering::Relaxed), 0);
    Ok(())
}

}