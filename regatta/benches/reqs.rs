use async_stream::stream;
use futures::{stream, Stream, StreamExt};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::time::Duration;
use tokio::time::{sleep, Instant};


/// Invokes count statements and returns a stream of their durations.
/// Note: this does *not* spawn a new thread.
/// It runs all async code on the caller's thread.
fn make_stream<'a>(session: &'a Session, statement: &'a String, count: usize)
    -> impl Stream<Item=Duration> + 'a {

    let concurrency_limit = 128;
    futures::stream::iter(0..count)
        .map(move |i| async move {
            let mut statement = statement.bind();
            let statement = statement.bind(0, i as i64).unwrap();
            let query_start = Instant::now();
            let result = session.execute(&statement);
            result.await.unwrap();
            query_start.elapsed()
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

async fn run_stream(session: std::sync::Arc<Session>, statement: std::sync::Arc<String>, count: usize) {
    let task = tokio::spawn(async move {
        let session = session.as_ref();
        let statement = statement.as_ref();
        let mut stream = make_stream(session, statement, count);
        while let Some(duration) = stream.next().await {}
    });
    task.await;
}

// Server abstraction that
pub struct Server {
    pub started: AtomicBool,
}

lazy_static! {
    static ref SERVER: RwLock<Server> = RwLock::new(Server::new());
}

async fn init_bench_server() {
    SERVER.write().unwrap().init_server().await;
}

async fn benchmark() {
    let count = 1000000;

    init_bench_server().await;

    let session = 1;// ... connect
    let session = std::sync::Arc::new(session);
    let statement = session.prepare("SELECT * FROM keyspace1.test WHERE pk = ?").unwrap().await.unwrap();
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