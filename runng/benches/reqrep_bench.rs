use criterion::{criterion_group, criterion_main, Criterion, ParameterizedBenchmark, Throughput};
use futures::future::Future;
use runng::{asyncio::*, *};
use std::time::Duration;

fn nng_reqrep(crit: &mut Criterion, url: &str) -> NngReturn {
    let url = url.to_owned();
    let parameters: Vec<usize> = vec![0, 128, 1024, 4 * 1024, 16 * 1024];

    let factory = Latest::default();
    let mut replier = factory.replier_open()?.listen(&url)?.create_async()?;
    let mut requester = factory.requester_open()?.dial(&url)?.create_async()?;

    let benchmark = ParameterizedBenchmark::new(
        format!("reqrep({})", url),
        move |bencher, param| bencher.iter(
            || -> NngReturn {
                let req_future = requester.send(msg::NngMsg::with_size(*param)?);
                let rep_future = replier.receive();
                let _request = rep_future.wait().unwrap()?;
                let rep_future = replier.reply(msg::NngMsg::with_size(*param)?);
                rep_future.wait().unwrap();
                let _reply = req_future.wait().unwrap()?;
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
    nng_reqrep(crit, "inproc://test").unwrap();
    nng_reqrep(crit, "ipc://test").unwrap();
    nng_reqrep(crit, "tcp://127.0.0.1:10287").unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);
