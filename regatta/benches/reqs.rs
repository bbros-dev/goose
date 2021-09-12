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

extern crate futures;
extern crate hyper;
extern crate num_cpus;
//extern crate service_fn;
//extern crate tokio_core;

use std::io;
use std::net::{self, SocketAddr};
use std::thread;
use std::time;

use futures::Future;
//use futures::stream::Stream;
use futures::sync::mpsc;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::server::conn::Http;
use hyper::Response;
//use service_fn::service_fn;
use tokio::net::TcpStream;
use tokio::reactor::Core;

const TEXT: &'static str = "Hello, World!";

fn init_real_server() {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let num_threads = num_cpus::get();

    let listener = net::TcpListener::bind(&addr).expect("failed to bind");
    println!("Listening on: {}", addr);

    let mut channels = Vec::new();
    for _ in 0..num_threads {
        let (tx, rx) = mpsc::unbounded();
        channels.push(tx);
        thread::spawn(|| worker(rx));
    }

    let mut next = 0;
    for socket in listener.incoming() {
        let socket = match socket {
            Ok(socket) => socket,
            // Ignore connection refused/aborted errors and directly continue
            // with the next request.
            Err(ref e) if connection_error(e) => continue,
            // Ignore socket errors like "Too many open files" on the OS
            // level. We need to sleep for a bit because the socket is not
            // removed from the accept queue in this case. A direct continue
            // would spin the CPU really hard with the same error again and
            // again.
            Err(_) => {
                let ten_millis = time::Duration::from_millis(10);
                thread::sleep(ten_millis);
                continue;
            }
        };
        channels[next]
            .unbounded_send(socket)
            .expect("worker thread died");
        next = (next + 1) % channels.len();
    }
}

// Represents one worker thread of the server that receives TCP connections from
// the main server thread.
fn worker(rx: mpsc::UnboundedReceiver<std::net::TcpStream>) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let http = hyper::server::conn::Http::new();

    let done = rx.for_each(move |socket| {
        let socket = match TcpStream::from_stream(socket, &handle) {
            Ok(socket) => socket,
            Err(error) => {
                println!(
                    "Failed to read TCP stream, ignoring connection. Error: {}",
                    error
                );
                return Ok(());
            }
        };
        let addr = match socket.peer_addr() {
            Ok(addr) => addr,
            Err(error) => {
                println!(
                    "Failed to get remote address, ignoring connection. Error: {}",
                    error
                );
                return Ok(());
            }
        };

        let hello = service_fn(|_req| {
            Ok(Response::<hyper::Body>::new()
                .headers_mut()
                .insert(CONTENT_LENGTH, TEXT.len() as u64)
                .insert(CONTENT_TYPE, TEXT.len() as u64)
                .with_header(ContentType::plaintext())
                .with_body(TEXT))
        });

        let connection = http.serve_connection(socket, hello)
            .map(|_| ())
            .map_err(move |err| println!("server connection error: ({}) {}", addr, err));

        handle.spawn(connection);
        Ok(())
    });
    match core.run(done) {
        Ok(_) => println!("Worker tokio core run ended unexpectedly"),
        Err(_) => println!("Worker tokio core run error."),
    };
}

/// This function defines errors that are per-connection. Which basically
/// means that if we get this error from `accept()` system call it means
/// next connection might be ready to be accepted.
///
/// All other errors will incur a timeout before next `accept()` is performed.
/// The timeout is useful to handle resource exhaustion errors like ENFILE
/// and EMFILE. Otherwise, could enter into tight loop.
fn connection_error(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::ConnectionRefused || e.kind() == io::ErrorKind::ConnectionAborted
        || e.kind() == io::ErrorKind::ConnectionReset
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
    //regatta::calibrate::init_real_server().await;
    init_real_server().await;
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
