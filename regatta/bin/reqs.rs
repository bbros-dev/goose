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
fn make_stream<'a>(session: &'a hyper::Client<hyper::client::HttpConnector>, statement: &'a hyper::Uri, count: usize)
    -> impl Stream + 'a {

    let concurrency_limit = 128;

    futures::stream::iter(0..count)
        .map( move |i| async move {
            let query_start = tokio::time::Instant::now();

            let statement = statement.clone();
            //Circa 13-15K req/sec
            //let (_parts, _body)  = session.get(statement).await.unwrap().into_parts();
            //let _ = session.get(statement).await.unwrap();
            query_start.elapsed()

            //futures::future::ready(query_start.elapsed()).await
            //futures::future::ready(query_start.elapsed())
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

async fn run_stream(session: std::sync::Arc<hyper::Client<hyper::client::HttpConnector>>, statement: std::sync::Arc<hyper::Uri>, count: usize) {
    let task = tokio::spawn(async move {
        let session = session.as_ref();
        let statement = statement.as_ref();
        let mut stream = make_stream(session, statement, count);
        while let Some(duration) = stream.next().await {}
    });
    task.await;
}

#[tokio::main]
async fn main(){
    let count = 10000;

    capacity_benchmark(count).await;
}

async fn capacity_benchmark(count: usize) {
    regatta::calibrate::init_real_server().await;

    let session1 = regatta::calibrate::client::Client::new().client;
    let session2 = regatta::calibrate::client::Client::new().client;
    let session1 = std::sync::Arc::new(session1);
    let session2 = std::sync::Arc::new(session2);
    // let statement = hyper::Request::builder()
    //     .method(hyper::Method::GET)
    //     .uri("http://localhost:8888")
    //     .body(hyper::Body::empty())
    //     .unwrap();
    let statement = "http://localhost:8888/".parse::<hyper::Uri>().unwrap();
    let statement = std::sync::Arc::new(statement);
    let benchmark_start = Instant::now();
    let thread_1 = run_stream(session1.clone(), statement.clone(), count / 4);
    let thread_2 = run_stream(session2.clone(), statement.clone(), count / 4);
    let thread_3 = run_stream(session1.clone(), statement.clone(), count / 4);
    let thread_4 = run_stream(session2.clone(), statement.clone(), count / 4);
    thread_1.await;
    thread_2.await;
    thread_3.await;
    thread_4.await;

    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    );
}
