use criterion::{criterion_group, criterion_main, Criterion, ParameterizedBenchmark, Throughput};
use futures::{future::Future, Stream};
use runng::{asyncio::*, protocol::*, *};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

fn nng_pushpull(crit: &mut Criterion, url: &str) -> NngReturn {
    let url = url.to_owned();
    //let parameters = vec![0, 16, 128, 1024, 4096];
    let parameters = vec![0];

    // Replier
    let factory = Latest::default();
    let mut pusher = factory.pusher_open()?.listen(&url)?.create_async()?;

    let mut puller = factory.puller_open()?.dial(&url)?.create_async_stream()?;

    let recv_count = Arc::new(AtomicUsize::new(0));
    let thread_count = recv_count.clone();
    let _thread = thread::spawn(move || {
        while thread_count.load(Ordering::Relaxed) == 0 {
            let push_future = pusher.send(msg::NngMsg::create().unwrap());
            push_future.wait().unwrap().unwrap();
        }
    });

    let benchmark = ParameterizedBenchmark::new(
        format!("pushpull({})", url),
        move |bencher, _param| bencher.iter(
            || -> NngReturn {
                // let msg = msg::MsgBuilder::default()
                //     .append_vec(&mut vec![0; *param])
                //     .build()?;
                let pull_future = puller.receive().unwrap().take(1).for_each(|_request| Ok(()));
                pull_future.wait()?;
                Ok(())
            }
            ),
        parameters)
        .sample_size(2)
        .warm_up_time(Duration::from_millis(5))
        .throughput(|_param| Throughput::Elements(1))
        .measurement_time(Duration::from_secs(1))
        // .with_function("function 2", |bencher, param| bencher.iter( || {
        //     true
        // }))
        ;
    crit.bench("group", benchmark);
    recv_count.fetch_add(1, Ordering::Relaxed);
    //thread.join();
    Ok(())
}

fn bench(crit: &mut Criterion) {
    nng_pushpull(crit, "inproc://test").unwrap();
    // nng_pushpull(crit, "ipc://test");
    // nng_pushpull(crit, "tcp://127.0.0.1:10287");
}

criterion_group!(benches, bench);
criterion_main!(benches);
