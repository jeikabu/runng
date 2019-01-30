use criterion::{criterion_group, criterion_main, Criterion, ParameterizedBenchmark, Throughput};
use futures::future::Future;
use runng::{asyncio::*, *};
use std::time::Duration;

fn latency(crit: &mut Criterion, url: &str) -> NngReturn {
    let url = url.to_owned();

    let parameters: Vec<usize> = vec![0, 128, 1024, 4 * 1024, 16 * 1024];

    let factory = Latest::default();
    let mut pusher = factory.pusher_open()?.listen(&url)?.create_async()?;
    let mut puller = factory.puller_open()?.dial(&url)?.create_async()?;

    let benchmark = ParameterizedBenchmark::new(
        format!("pushpull({})", url),
        move |bencher, param| bencher.iter(
            || -> NngReturn {
                let push_future = pusher.send(msg::NngMsg::with_size(*param)?);
                let pull_future = puller.receive();
                push_future.wait().unwrap()?;
                pull_future.wait()?;
                Ok(())
            }
            ),
        parameters)
        .warm_up_time(Duration::from_millis(5))
        .throughput(|_param| Throughput::Elements(1))
        .measurement_time(Duration::from_secs(1))
        // .with_function("function 2", |bencher, param| bencher.iter( || {
        //     true
        // }))
        ;
    crit.bench("group", benchmark);
    Ok(())
}

fn bench(crit: &mut Criterion) {
    latency(crit, "inproc://test").unwrap();
    latency(crit, "ipc://test").unwrap();
    latency(crit, "tcp://127.0.0.1:10287").unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);
