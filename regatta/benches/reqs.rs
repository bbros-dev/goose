use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

//use criterion::async_executor::FuturesExecutor;
//use tokio::runtime::Runtime;

//use async_stream::stream;
use futures::{Stream, StreamExt};
//use hyper::body::Bytes;
use lazy_static::lazy_static;
//use rand::distributions::{Distribution, Uniform};
//use std::time::Duration;
use tokio::time::Instant;

lazy_static! {
    static ref URL: hyper::Uri = hyper::Uri::from_static("http://localhost:8888");
}

/// Invokes client get URL, return a stream of response durations.
/// Note: Does *not* spawn new threads. Requests are concurrent not parallel.
/// Async code runs on the caller thread.
///    -> impl Stream<Item=Duration> + 'a {
fn make_stream<'a>(
    session: &'a hyper::Client<hyper::client::HttpConnector>,
    statement: &'a hyper::Uri,
    count: usize,
) -> impl futures::Stream + 'a {
    let concurrency_limit = 1024;

    futures::stream::iter(0..count)
        .map(move |_| async move {
            let statement = statement.clone();
            let query_start = tokio::time::Instant::now();
            let _response = session.get(statement).await;
            // let (_parts, _body)  = response.unwrap().into_parts();
            query_start.elapsed()
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

fn make_stream_tls<'a>(
    session: &'a hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    statement: &'a hyper::Uri,
    count: usize,
) -> impl Stream + 'a {
    let concurrency_limit = 128;

    futures::stream::iter(0..count)
        .map(move |_| async move {
            let statement = statement.clone();
            let query_start = tokio::time::Instant::now();
            let _response = session.get(statement).await;
            // let (_parts, _body)  = response.unwrap().into_parts();
            query_start.elapsed()
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

async fn run_stream(
    session: hyper::Client<hyper::client::HttpConnector>,
    statement: &'static URL,
    count: usize,
) {
    let task = tokio::spawn(async move {
        let session = &session;
        //let statement = &statement;
        let mut stream = make_stream(session, statement, count);
        while let Some(duration) = stream.next().await {}
    });
    task.await;
}

async fn run_stream_tls(
    session: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    statement: &'static URL,
    count: usize,
) {
    let task = tokio::spawn(async move {
        let session = &session;
        let statement = &statement;
        let mut stream = make_stream_tls(session, statement, count);
        while let Some(duration) = stream.next().await {}
    });
    task.await;
}

async fn capacity(count: usize) {
    regatta::calibrate::init_real_server().await;
    let session = regatta::calibrate::client::Client::new().client;
    //let session = std::sync::Arc::new(session);
    //let statement = "http://localhost:8888".parse::<hyper::Uri>().unwrap();
    let statement = &URL;
    //let statement = std::sync::Arc::new(statement);
    let benchmark_start = Instant::now();
    let thread_1 = tokio::task::spawn(run_stream(session.clone(), statement, count / 2));
    //let thread_2 = tokio::task::spawn(run_stream(session.clone(), statement.clone(), count / 2));
    thread_1.await;
    //thread_2.await;

    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    );
}

fn calibrate_limit(c: &mut Criterion) {
    println!("Running on thread {:?}", std::thread::current().id());

    let mut group = c.benchmark_group("Calibrate");
    let count = 100000;
    let tokio_executor = tokio::runtime::Runtime::new().expect("initializing tokio runtime");
    group.bench_with_input(
        BenchmarkId::new("calibrate-limit", count),
        &count,
        |b, &s| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&tokio_executor).iter(|| capacity(count));
        },
    );

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Protobuf));
    targets = calibrate_limit
}

criterion_main!(benches);
