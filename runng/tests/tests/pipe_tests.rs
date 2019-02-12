//#![cfg(feature = "pipes")]

use crate::common::{get_url, sleep_fast};
use runng::{factory::latest::ProtocolFactory, msg::NngMsg, pipe::*, socket::*};
use runng_sys::{nng_pipe, nng_pipe_ev, nng_pipe_ev::*};
use std::sync::atomic::{AtomicUsize, Ordering};

static NUM_ADDPRE: AtomicUsize = AtomicUsize::new(0);
static NUM_ADDPOST: AtomicUsize = AtomicUsize::new(0);
static NUM_REMPOST: AtomicUsize = AtomicUsize::new(0);
static NUM_BAD: AtomicUsize = AtomicUsize::new(0);

extern "C" fn notify_callback(_pipe: nng_pipe, event: i32, _arg: PipeNotifyCallbackArg) {
    match nng_pipe_ev::try_from(event) {
        Ok(NNG_PIPE_EV_ADD_PRE) => NUM_ADDPRE.fetch_add(1, Ordering::Relaxed),
        Ok(NNG_PIPE_EV_ADD_POST) => NUM_ADDPOST.fetch_add(1, Ordering::Relaxed),
        Ok(NNG_PIPE_EV_REM_POST) => NUM_REMPOST.fetch_add(1, Ordering::Relaxed),
        _ => NUM_BAD.fetch_add(1, Ordering::Relaxed),
    };
}

#[test]
fn notify() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    [
        NNG_PIPE_EV_ADD_PRE,
        NNG_PIPE_EV_ADD_POST,
        NNG_PIPE_EV_REM_POST,
    ]
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
fn dialer_listener() -> runng::Result<()> {
    let url = get_url();

    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.sendmsg(NngMsg::create()?)?;
    let msg = rep.recvmsg()?;
    let rep_pipe = msg.get_pipe().unwrap();
    rep.sendmsg(NngMsg::create()?)?;
    let msg = req.recvmsg()?;
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
