#[macro_use]
extern crate criterion;
extern crate runng;

use criterion::{
    Criterion,
    ParameterizedBenchmark,
    Throughput,
};
use futures::{
    future,
    future::Future,
    Stream,
};
use runng::{
    *,
    protocol::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};


fn nng_reqrep(crit: &mut Criterion, url: &str) -> NngReturn {
    let url = url.to_owned();
    let parameters = vec![0, 1024];

    // Replier
    let factory = Latest::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;
    let rep_future = rep_ctx.receive()
        .for_each(|_request|{
            let msg = msg::NngMsg::new().unwrap();
            rep_ctx.reply(msg).wait().unwrap();
            Ok(())
        });

    let benchmark = ParameterizedBenchmark::new(
        format!("reqrep({})", url),
        move |bencher, param| bencher.iter_with_setup(
            || -> NngResult<Box<AsyncRequestContext>> {
                let factory = Latest::default();
                let requester = factory.requester_open()?
                    .dial(&url)?
                    .create_async_context();
                requester
            },
            |mut requester| -> NngReturn {
                let mut data = vec![0; *param];
                let msg = msg::MsgBuilder::default()
                    .append_vec(&mut data)
                    .build()?;
                let req_future = requester?.send(msg);
                req_future.wait().unwrap()?;
                Ok(())
        }),
        parameters)
        .sample_size(2)
        .warm_up_time(Duration::from_millis(500))
        .throughput(|param| Throughput::Elements(1))
        .measurement_time(Duration::from_millis(500))
        
        // .with_function("function 2", |bencher, param| bencher.iter( || {
        //     true
        // }))
        ;
    crit.bench("group", benchmark);
    
    Ok(())
}

fn bench(crit: &mut Criterion) {
    //nng_reqrep(crit, "inproc://test");
    nng_reqrep(crit, "ipc://test");
    //nng_reqrep(crit, "tcp://127.0.0.1:10287");
}

criterion_group!(benches, bench);
criterion_main!(benches);