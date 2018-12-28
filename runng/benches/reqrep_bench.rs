#[macro_use]
extern crate criterion;
extern crate runng;

use criterion::{
    Criterion,
    ParameterizedBenchmark,
    Throughput,
};
use futures::{
    future::Future,
    Stream,
};
use runng::{
    *,
    protocol::*,
};
use std::{
    thread,
    time::Duration,
};

fn request_reply(requester: &mut AsyncRequestContext, request_bytes: usize) -> NngReturn {
    let mut data = vec![0; request_bytes];
    let msg = msg::MsgBuilder::default()
        .append_vec(&mut data)
        .build()?;
    let req_future = requester.send(msg);
    req_future.wait().unwrap()?;
    Ok(())
}

fn nng_reqrep(crit: &mut Criterion, url: &str) -> NngReturn {
    let url = url.to_owned();
    let parameters = vec![0, 16, 128, 1024, 4096];

    // Replier
    let factory = Latest::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;
    thread::spawn(move || {
        let rep_future = rep_ctx.receive()
            .for_each(|_request|{
                let msg = msg::NngMsg::new().unwrap();
                rep_ctx.reply(msg).wait().unwrap();
                Ok(())
            });
        rep_future.wait();
    });
    
    let mut requester = factory.requester_open()?
        .dial(&url)?
        .create_async_context()?;

    let benchmark = ParameterizedBenchmark::new(
        format!("reqrep({})", url),
        move |bencher, param| bencher.iter(
            || request_reply(&mut requester, *param).unwrap()
            ),
        parameters)
        .sample_size(4)
        .warm_up_time(Duration::from_millis(1000))
        .throughput(|_param| Throughput::Elements(1))
        .measurement_time(Duration::from_secs(4))
        // .with_function("function 2", |bencher, param| bencher.iter( || {
        //     true
        // }))
        ;
    crit.bench("group", benchmark);
    
    Ok(())
}

fn bench(crit: &mut Criterion) {
    nng_reqrep(crit, "inproc://test").unwrap();
    nng_reqrep(crit, "ipc://test").unwrap();
    nng_reqrep(crit, "tcp://127.0.0.1:10287").unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);