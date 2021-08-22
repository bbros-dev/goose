use criterion::{BenchmarkId, black_box, criterion_group, criterion_main, Criterion};
//use criterion::async_executor::FuturesExecutor;
use tokio::runtime::Runtime;

use async_stream::stream;
use futures::{stream, Stream, StreamExt};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::time::Duration;
use tokio::time::{sleep, Instant};


/// Invokes count statements and returns a stream of their durations.
/// Note: this does *not* spawn a new thread.
/// It runs all async code on the caller's thread.
//    -> impl Stream<Item=Duration> + 'a {
fn make_stream<'a>(session: &'a hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>, statement: &'a hyper::Uri, count: usize)
    -> impl Stream + 'a {

    let concurrency_limit = 128;

    futures::stream::iter(0..count)
        .map( move |i| async move {
            //let session = session.clone();
            //let statement = statement.clone();
            let query_start = tokio::time::Instant::now();

            let statement = statement.clone();
            //let (_parts, _body)  = session.request(*statement).await.unwrap().into_parts();
            //let uri = "http://localhost:8888".parse().unwrap();
            let resp = session.get(statement).await;
            let (_parts, _body)  = resp.unwrap().into_parts();
            query_start.elapsed()

            //futures::future::ready(query_start.elapsed()).await
            //futures::future::ready(query_start.elapsed())
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

async fn run_stream(session: std::sync::Arc<hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>>, statement: std::sync::Arc<hyper::Uri>, count: usize) {
    let task = tokio::spawn(async move {
        let session = session.as_ref();
        let statement = statement.as_ref();
        let mut stream = make_stream(session, statement, count);
        while let Some(duration) = stream.next().await {}
    });
    task.await;
}

async fn capacity_benchmark(count: usize) {
    regatta::calibrate::init_real_server().await;
    let session = regatta::calibrate::client::Client::new().client;
    let session = std::sync::Arc::new(session);
    // let statement = hyper::Request::builder()
    //     .method(hyper::Method::GET)
    //     .uri("http://localhost:8888")
    //     .body(hyper::Body::empty())
    //     .unwrap();
    let statement = "http://localhost:8888".parse::<hyper::Uri>().unwrap();
    let statement = std::sync::Arc::new(statement);
    let benchmark_start = Instant::now();
    let thread_1 = run_stream(session.clone(), statement.clone(), count / 2);
    let thread_2 = run_stream(session.clone(), statement.clone(), count / 2);
    thread_1.await;
    thread_2.await;

    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    );
}

fn calibrate_limit(c: &mut Criterion) {
    let count = 10000;
    let tokio_executor = tokio::runtime::Runtime::new()
                                .expect("initializing tokio runtime");
    c.bench_with_input(BenchmarkId::new("calibrate-limit", count), &count, |b, &s| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(&tokio_executor).iter(|| capacity_benchmark(count));
    });
}

criterion_group!(benches, calibrate_limit);

criterion_main!(benches);
