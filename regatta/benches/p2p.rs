// Peer-to-Peer Benchmarks
// The p2p component performs these functions
// - Discover/forget Regatta peers
// - Connect/disconnect Regatta peers
// - Distribute VCR file to Regatta peers
//
// This benchmark records the performance of:
// 1. Launch Regatta triple (currently `p2p`)
// 2. Distribute VCR cassette
// 3. Read back distributed VCR cassette (correctness check)
// 4. Shutdown Regatta triple

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::Stream;
use hyper::{body::HttpBody as _, Client};
use std::io::Write;

//
// Reference
// https://pkolaczk.github.io/benchmarking-cassandra-with-rust-streams/
//
/// Invokes count statements and returns a stream of their durations.
/// Note: this does *not* spawn a new thread.
/// It runs all async code on the caller's thread.
// not tested: use futures::stream::Iter;
// not tested: use tokio_stream::StreamExt;
use futures::{stream, StreamExt};

//fn calibrate_ceiling<'a>(client: &'a Session, uri: &'a PreparedStatement, count: usize)
fn calibrate_ceiling<'a>(count: i32) -> impl Stream<Item = std::time::Duration> + 'a {
    let concurrency = 128;
    futures::stream::iter(0..count)
        .map(move |i| async move {
            //let mut request = request.bind();
            //let request = request.bind(0, i as i64).unwrap();
            let query_start = tokio::time::Instant::now();
            //let result = session.execute(&request);
            //result.await.unwrap();
            query_start.elapsed()
        })
        // This will run up to `concurrency` futures at a time:
        .buffer_unordered(concurrency)
}

async fn ceiling_benchmark() {
    let count: i32 = 1000000;

    // Connect to the web server and prepare the request:
    //let session = // ...
    //let req = session.get(/** URL */).unwrap().await.unwrap();
    //let mut stream = calibrate_ceiling(&session, &req, count);
    let mut stream = calibrate_ceiling(count);

    // Process the received durations:
    let benchmark_start = tokio::time::Instant::now();
    while let Some(duration) = stream.next().await {
        // ... optionally compute durations statistics
    }
    // println!(
    //     "Throughput: {:.1} request/s",
    //     1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    // );
}

// Extend `calibrate_ceiling` with HTTP client request logic.
// Benchmark alternative client performance here.
//
// Implementation references:
// https://blog.yoshuawuyts.com/parallel-stream/ and https://github.com/async-rs/parallel-stream
// https://stackoverflow.com/a/51047786 and https://gist.github.com/lu4nm3/b8bca9431cdcf19d73040ada13387e58
// https://www.zupzup.org/rust-http-testing/
// http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
// https://www.philipdaniels.com/blog/2019/async-std-demo1/
// https://www.programmersought.com/article/19657332752/
// https://kerkour.com/blog/rust-worker-pool/
// https://gendignoux.com/blog/2020/12/17/rust-async-type-system-limits.html

//fn calibrate_overhead<'a>(client: &'a Session, uri: &'a PreparedStatement, count: usize)
fn calibrate_overhead<'a>(count: i32) -> impl Stream<Item = std::time::Duration> + 'a {
    //let server = calibration_server()
    //let mut client = Client::new();
    //let uri = "http://localhost:3000/".parse()?;
    let concurrency = 128;
    futures::stream::iter(0..count)
        .map(move |i | async move {
            //let request = request.bind(0, i as i64).unwrap();
            let query_start = tokio::time::Instant::now();
            //let response = client.get(uri).await?;
            // Stream the body as we get it;
            //while let Some(next) = response.data().await {};
            query_start.elapsed()
            // Add time to histogram log.
        })
        // Run up to `concurrency` futures at a time:
        .buffer_unordered(concurrency)
}

async fn overhead_benchmark() {
    let count: i32 = 1000000;

    // Connect to the web server and prepare the request:
    //let session = // ...
    //let req = session.get(/** URL */).unwrap().await.unwrap();
    //let mut stream = calibrate_ceiling(&session, &req, count);
    let mut stream = calibrate_overhead(count);

    // Process the received durations:
    let benchmark_start = tokio::time::Instant::now();
    while let Some(duration) = stream.next().await {
        // ... optionally compute durations statistics
    }
    // println!(
    //     "Throughput: {:.1} request/s",
    //     1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    // );
}

fn calibrate(c: &mut Criterion) {
    let mut group = c.benchmark_group("calibrate");
    group.bench_function("ceiling_benchmark", |b| {
        b.iter(|| black_box(ceiling_benchmark()))
    });

    group.bench_function("overhead_benchmark", |b| {
        b.iter(|| black_box(overhead_benchmark()))
    });

    // group.bench_function("limit_benchmark", |b| {
    //     b.iter(|| limit_benchmark())
    // });

    // group.bench_function("drift_benchmark", |b| {
    //     b.iter(|| drift_benchmark())
    // });

    group.finish();
}

#[cfg(feature = r#"""#)]
fn bench_setup_cassette() {
    let w = std::fs::File::create("./recipes.json").unwrap();
    writeln!(&w, r#"[{"id":0,"name":" Coffee","ingredients":"Coffee","instructions":"Make Coffee","public":true},{"id":1,"name":" Tea","ingredients":"Tea, Water","instructions":"Boil Water, add tea","public":false},{"id":2,"name":" Carrot Cake","ingredients":"Carrots, Cake","instructions":"Make Carrot Cake","public":true}]"#).unwrap();
}

fn bench_read_cassette() {
    regatta::p2p::cassette::read_local_recipes();
}

fn cassettes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cassettes");

    // bench_setup_cassette();
    group.bench_function("vcr-read", |b| b.iter(|| bench_read_cassette()));

    group.finish();
}

criterion_group!(benches, cassettes, calibrate);

criterion_main!(benches);
