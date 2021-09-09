use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use tracing::{self, debug, error, info, span, trace, warn, Instrument as _, Level, Span};
//use tracing_subscriber::{filter::EnvFilter, reload::Handle};
use tracing::instrument;
//use tracing_attributes::instrument;

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
    static ref URL: hyper::Uri = hyper::Uri::from_static("http://127.0.0.1:8888");
}

/// Invokes client get URL, return a stream of response durations.
/// Note: Does *not* spawn new threads. Requests are concurrent not parallel.
/// Async code runs on the caller thread.
///    -> impl Stream<Item=Duration> + 'a {
#[instrument]
fn make_stream<'a>(
    session: &'a hyper::Client<hyper::client::HttpConnector>,
    statement: &'a hyper::Uri,
    count: usize,
) -> impl futures::Stream + 'a {
    let concurrency_limit = 512;

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

async fn run_stream_ct(
    session: hyper::Client<hyper::client::HttpConnector>,
    statement: &'static URL,
    count: usize,
) {
    // Construct a local task set that can run `!Send` futures.
    //let local = tokio::task::LocalSet::new();
    // Run the local task set.
    //local.run_until(async move {
        //let task = tokio::task::spawn_local(async move {
            //let session = &session;
            //let statement = &statement;
            debug!("About to make stream");
            let mut stream = make_stream(&session, statement, count);
            while let Some(duration) = stream.next().await {
                debug!("Stream next polled.");
            }
        //}).await.unwrap();
    //}).await;
    //task.await;
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
#[instrument]
async fn capacity(count: usize) {
    debug!("About to init server");
    regatta::calibrate::init_real_server().await;
    debug!("About to init client");
    let session = regatta::calibrate::client::Client::new().client;
    //let session = std::sync::Arc::new(session);
    //let statement = "http://localhost:8888".parse::<hyper::Uri>().unwrap();
    let statement = &URL;
    //let statement = std::sync::Arc::new(statement);
    let benchmark_start = tokio::time::Instant::now();
    debug!("Client: About to spawn blocking (Tokio)");
    tokio::task::spawn_blocking( move || {
        let rt = tokio::runtime::Handle::current();
        debug!("Client: About to block on (Tokio)");
        rt.block_on( async {
            let local = tokio::task::LocalSet::new();
            debug!("Client: About to run until (local set)");
            local.run_until(async move {
                //let task = tokio::task::spawn_local(async move {
                debug!("Client: About to spawn local (Tokio)");
                tokio::task::spawn_local(run_stream_ct(session.clone(), statement, count / 2)).await.expect("Tokio spawn local (streams for clients)");
            }).await;
        });
    }).await.unwrap();
        //let thread_2 = tokio::task::spawn(run_stream(session.clone(), statement.clone(), count / 2));
    //thread_1.await;
    //thread_2.await;

    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    );
}

fn calibrate_limit(c: &mut Criterion) {
    tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init().expect("Tracing subscriber in benchmark");
    debug!("Running on thread {:?}", std::thread::current().id());
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
