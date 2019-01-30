//#![cfg(feature = "pipes")]

use crate::common::{get_url, sleep_fast};
use runng::pipe::*;
use runng::*;
use runng_sys::nng_pipe;
use std::sync::atomic::{AtomicUsize, Ordering};

static NUM_ADDPRE: AtomicUsize = AtomicUsize::new(0);
static NUM_ADDPOST: AtomicUsize = AtomicUsize::new(0);
static NUM_REMPOST: AtomicUsize = AtomicUsize::new(0);
static NUM_BAD: AtomicUsize = AtomicUsize::new(0);

extern "C" fn notify_callback(_pipe: nng_pipe, event: i32, _arg: PipeNotifyCallbackArg) {
    match PipeEvent::from_i32(event) {
        Some(PipeEvent::AddPre) => NUM_ADDPRE.fetch_add(1, Ordering::Relaxed),
        Some(PipeEvent::AddPost) => NUM_ADDPOST.fetch_add(1, Ordering::Relaxed),
        Some(PipeEvent::RemPost) => NUM_REMPOST.fetch_add(1, Ordering::Relaxed),
        _ => NUM_BAD.fetch_add(1, Ordering::Relaxed),
    };
}

#[test]
fn notify() -> NngReturn {
    let url = get_url();

    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    [PipeEvent::AddPre, PipeEvent::AddPost, PipeEvent::RemPost]
        .iter()
        .for_each(|event| {
            rep.socket()
                .notify(*event, notify_callback, std::ptr::null_mut())
                .unwrap()
        });
    {
        let _ = factory.requester_open()?.dial(&url)?;
        // Give all notifications a chance to be delivered (especially Linux Travis CI)
        sleep_fast();
    }

    assert_eq!(NUM_ADDPRE.load(Ordering::Relaxed), 1);
    assert_eq!(NUM_ADDPOST.load(Ordering::Relaxed), 1);
    assert_eq!(NUM_REMPOST.load(Ordering::Relaxed), 1);
    assert_eq!(NUM_BAD.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn dialer_listener() -> NngReturn {
    let url = get_url();

    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::create()?)?;
    let msg = rep.recv()?;
    let rep_pipe = msg.get_pipe().unwrap();
    rep.send(msg::NngMsg::create()?)?;
    let msg = req.recv()?;
    let req_pipe = msg.get_pipe().unwrap();

    unsafe {
        // Requester pipe
        req_pipe.socket().unwrap();
        req_pipe.dialer().unwrap();
        assert!(req_pipe.listener().is_none());

        // Replier pipe
        rep_pipe.socket().unwrap();
        assert!(rep_pipe.dialer().is_none());
        rep_pipe.listener().unwrap();
    }

    Ok(())
}
