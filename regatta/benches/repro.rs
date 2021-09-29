/// All bin and lib files are empty:
//
/// Cargo.toml: start
//
// [package]
// name = "regatta"
// version = "0.1.0"
// edition = "2018"

// [dependencies]
// async-stream = "0.3"
// async-trait = "0.1"
// futures = "0.3.15"
// futures-util = "0.3.17"
// hyper = { version = "0.14", features = ["full"] }
// lazy_static = "1.4.0"
// num_cpus = "1.13.0"
// tokio = { version = "^1.8", features = [
//     "fs",
//     "io-std",
//     "io-util",
//     "macros",
//     "net",
//     "rt",
//     "rt-multi-thread",
//     "signal",
//     "sync",
//     "time"
// ] }
// tokio-stream = "0.1"
// tracing = "0.1.26"
// tracing-subscriber = "0.2.20"
// url = "2.2"

// [dev-dependencies]
// cargo-criterion = "1.0.1"
// criterion = {version = "0.3", features = ["async_tokio", "html_reports"]}
// critcmp = "0.1.7"
// lazy_static = "=1.4.0"
// pprof = { version = "0.5", features = ["criterion", "flamegraph", "protobuf"] }

// [profile.dev]
// lto = false

// [target.x86_64-unknown-linux-gnu]
// linker = "/usr/bin/clang"
// rustflags = [
//     "-Clink-arg=-fuse-ld=lld", "-Zshare-generics=y"
// ]

// [[bench]]
// name = "repro"
// harness = false

/// Cargo.toml: end
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use tracing::instrument;
use tracing::{self, debug, error, info, span, trace, warn, Instrument as _, Level, Span};

use async_trait::async_trait;
use futures::StreamExt;
use lazy_static::lazy_static;
use std::convert::TryInto;

lazy_static! {
    static ref URL: surf::Url = surf::Url::parse("http://127.0.0.1:8888").expect("URL");
}

const URI: &str = "https://127.0.0.1";

extern crate futures;
extern crate hyper;
use hyper::{
    body::to_bytes, body::HttpBody as _, client::HttpConnector, Body, Client as HyperClient,
    Method, Request,
};
extern crate num_cpus;

use std::io;
use std::net::SocketAddr;
use std::time;

use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use std::io::{Read, Write};
use tokio::net::TcpStream;

static HELLO: &[u8] = b"Hello World!";

static RESPONSE: &str = "HTTP/1.1 200 OK
Content-Type: text/html
Connection: keep-alive
Content-Length: 13

Hello World!
";

fn is_double_crnl(window: &[u8]) -> bool {
    window.len() >= 4
        && (window[0] == '\r' as u8)
        && (window[1] == '\n' as u8)
        && (window[2] == '\r' as u8)
        && (window[3] == '\n' as u8)
}

#[async_trait]
pub trait HttpClient: Send + Sync + Clone + 'static {
    async fn get_url(&self) -> hyper::Uri;
}

#[derive(Clone, Debug)]
pub struct Client {
    pub client: HyperClient<HttpConnector>,
    //pub client_tls: HyperClient<HttpsConnector<HttpConnector>>,
    pub addresses: std::vec::Vec<std::net::SocketAddr>
}

impl Client {
    pub fn new() -> Self {
        Self {
            //client_tls: https_client(),
            client: http_client(),
            addresses: vec![]
        }
    }

    fn get_url(&self) -> hyper::Uri {
        // HTTPS requires picking a TLS implementation, so give a better
        // warning if the user tries to request an 'https' URL.
        //let url =
        URI.to_owned().parse::<hyper::Uri>().unwrap()
        // if url.scheme_str() != Some("http") {
        //     println!("This example only works with 'http' URLs.");
        //     return "http://127.0.0.1".parse::<hyper::Uri>().unwrap();
        // }
        //url
    }
}

fn http_client() -> HyperClient<hyper::client::HttpConnector> {
    HyperClient::new()
}

enum Msg {
    Echo(String),
    Client { client: HyperClient<hyper::client::HttpConnector>, addresses: std::vec::Vec<std::net::SocketAddr> },
}

// This is a very neat pattern for stopping a thread...
// After starting the thread:
//     drop(worker.take());
//
// https://matklad.github.io/2018/03/03/stopping-a-rust-worker.html
fn spawn_server(message: std::sync::Arc<std::sync::Mutex<Option<Msg>>> ) -> std::sync::mpsc::Sender<Msg> {
    // let client = Msg::Client{ client: http_client(), addresses: vec![]};
    // let message: std::sync::Arc<std::sync::Mutex<Option<Msg>>> = std::sync::Arc::new(std::sync::Mutex::new(Some(client)));
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            match msg {
                Msg::Echo(_txt) => {
                    let l = message.lock().unwrap().take().unwrap();
                    if let Msg::Client{client,mut addresses} = l {
                        let listener = mio::net::TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
                        let address = listener.local_addr().unwrap();
                        addresses.push(address);
                    }
                    init_mio_server();
                },
                Msg::Client { client,addresses} => {
                    debug!("Do we need this?")
                }
            }
        }
        println!("The server has stopped!");
    });
    tx
}

// This is a lean TCP server for responding with Hello World! to a request.
// https://github.com/sergey-melnychuk/mio-tcp-server
use mio::{Events, Poll, Interest, Token};
fn init_mio_server() {
    // Get assigned free port. Store in a struct.
    let address = "127.0.0.1:8888";
    // let listener = mio::net::TcpListener::bind(&address.parse().unwrap()).unwrap();
    let mut listener = reuse_mio_listener(&address.parse().unwrap()).expect("Could not bind to addr");
    let mut poll = mio::Poll::new().unwrap();
    poll.registry().register(
        &mut listener,
        mio::Token(0),
        mio::Interest::READABLE
    )
    .unwrap();

    let mut counter: usize = 0;
    let mut sockets: std::collections::HashMap<mio::Token, mio::net::TcpStream> =
        std::collections::HashMap::new();
    let mut requests: std::collections::HashMap<mio::Token, Vec<u8>> =
        std::collections::HashMap::new();
    let mut buffer = [0 as u8; 1024];

    let mut events = mio::Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            match event.token() {
                mio::Token(0) => loop {
                    match listener.accept() {
                        Ok((mut socket, _)) => {
                            counter += 1;
                            let token = mio::Token(counter);

                            poll.registry().register(
                                &mut socket,
                                token,
                                mio::Interest::READABLE
                            )
                            .unwrap();

                            sockets.insert(token, socket);
                            requests.insert(token, Vec::with_capacity(8192));
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(_) => break,
                    }
                },
                token if event.is_readable() => {
                    loop {
                        let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                        match read {
                            Ok(0) => {
                                sockets.remove(&token);
                                break;
                            }
                            Ok(n) => {
                                let req = requests.get_mut(&token).unwrap();
                                for b in &buffer[0..n] {
                                    req.push(*b);
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(_) => break,
                        }
                    }

                    let ready = requests
                        .get(&token)
                        .unwrap()
                        .windows(4)
                        .find(|window| is_double_crnl(*window))
                        .is_some();

                    if ready {
                        //let mut socket = sockets.get_mut(&token).unwrap();
                        poll.registry().reregister(
                            sockets.get_mut(&token).unwrap(),
                            token,
                            mio::Interest::WRITABLE
                        )
                        .unwrap();
                    }
                }
                token if event.is_writable() => {
                    requests.get_mut(&token).unwrap().clear();
                    sockets
                        .get_mut(&token)
                        .unwrap()
                        .write_all(RESPONSE.as_bytes())
                        .unwrap();

                    // Re-use existing connection ("keep-alive") - switch back to reading
                    //let &socket = sockets.get(&token).unwrap();
                    poll.registry().reregister(
                        sockets.get_mut(&token).unwrap(),
                        token,
                        mio::Interest::READABLE
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
    }
}

/// This is the main function, evolved from this gist:
/// https://gist.github.com/klausi/f94b9aff7d36a1cb4ebbca746f0a099f
#[instrument]
async fn init_real_server() {
    let addr: SocketAddr = ([127, 0, 0, 1], 8888).into();
    let mut channels = Vec::new();
    let num_threads = num_cpus::get();
    for _i in 0..num_threads {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        channels.push(tx);
        tokio::spawn(async move {
            worker(rx).await;
        });
    }
    let listener = reuse_listener(&addr).await.expect("couldn't bind to addr");
    println!("Listening on address: {}", addr);
    let mut next = 0;
    let accepting = async move {
        loop {
            let socket = listener.accept().await;
            debug!("Entered while let socket accept loop");
            match socket.as_ref() {
                Ok((stream, _)) => {
                    debug!("Stream processing...");
                    stream
                        .set_nodelay(true)
                        .expect("Set no delay on TCP stream.");
                    stream.readable().await.expect("A readable TCP stream.");
                    let (stream, _) = socket.unwrap();
                    debug!("Sending to channel #: {}", next);
                    // Skip when socket closed
                    if !channels[next].is_closed() {
                        let v = match channels[next].send(stream) {
                            Ok(v) => {
                                next = (next + 1) % channels.len();
                            }
                            Err(e) => return warn!("Send failed ({})", e),
                        };
                    }
                }
                Err(e) if connection_error(e) => continue,
                // Ignore socket errors like "Too many open files" on the OS
                // level. We need to sleep for a bit because the socket is not
                // removed from the accept queue in this case. A direct continue
                // would spin the CPU really hard with the same error again and
                // again.
                Err(_) => {
                    let ten_millis = time::Duration::from_millis(10);
                    std::thread::sleep(ten_millis);
                    continue;
                }
            }
        }
    };
    let handle = tokio::spawn(async { accepting.await });
    debug!("Exiting init_real_server");
}

// Set content-type and content-length headers and return response.
async fn hello(
    _: hyper::Request<hyper::Body>,
) -> std::result::Result<hyper::Response<hyper::Body>, std::convert::Infallible> {
    let mut resp = hyper::Response::new(hyper::Body::from(HELLO));
    let mut head = resp.headers_mut();
    head.insert(
        CONTENT_LENGTH,
        hyper::header::HeaderValue::from_str("12").unwrap(),
    );
    head.insert(
        CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("text/plain"),
    );
    Ok(resp)
}

/// Represents one worker thread of the server that receives TCP connections from
/// the main server thread.
/// This is the `worker` function from the gist related to `init_real_server`.
async fn worker(mut rx: tokio::sync::mpsc::UnboundedReceiver<tokio::net::TcpStream>) {
    debug!("Entering worker function.");
    while let Some(socket) = rx.recv().await {
        debug!("Channel receiver polled.");
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
    debug!("The worker has stopped!");
}

async fn process(socket: TcpStream) {
    debug!(
        "Serving to {:?} using thread {:?}",
        socket.peer_addr(),
        std::thread::current().id()
    );

    let mut http = hyper::server::conn::Http::new();
    http.http1_only(true);
    http.max_buf_size(8192);
    http.pipeline_flush(true);
    let serve = http.serve_connection(socket, hyper::service::service_fn(hello));
    if let Err(e) = serve.await {
        debug!("server connection error: {}", e);
    }
}

async fn reuse_listener(
    addr: &std::net::SocketAddr,
) -> Result<tokio::net::TcpListener, std::convert::Infallible> {
    let builder = match *addr {
        std::net::SocketAddr::V4(_) => tokio::net::TcpSocket::new_v4().expect("TCP v4"),
        std::net::SocketAddr::V6(_) => tokio::net::TcpSocket::new_v6().expect("TCP v6"),
    };
    builder.set_reuseport(true).expect("Reusable port");
    builder.set_reuseaddr(true).expect("Reusable address");
    builder.bind(*addr).expect("TCP socket");
    Ok(builder.listen(1024).expect("TCP listener"))
}

fn reuse_mio_listener(
    addr: &std::net::SocketAddr,
) -> Result<mio::net::TcpListener, std::convert::Infallible> {
    let builder = match *addr {
        std::net::SocketAddr::V4(_) => mio::net::TcpSocket::new_v4().expect("TCP v4"),
        std::net::SocketAddr::V6(_) => mio::net::TcpSocket::new_v6().expect("TCP v6"),
    };
    builder.set_reuseport(true).expect("Reusable port");
    builder.set_reuseaddr(true).expect("Reusable address");
    builder.bind(*addr).expect("TCP socket");
    Ok(builder.listen(1024).expect("TCP listener"))
}

/// This function defines errors that are per-connection. Which basically
/// means that if we get this error from `accept()` system call it means
/// next connection might be ready to be accepted.
///
/// All other errors will incur a timeout before next `accept()` is performed.
/// The timeout is useful to handle resource exhaustion errors like ENFILE
/// and EMFILE. Otherwise, could enter into tight loop.
fn connection_error(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::ConnectionRefused
        || e.kind() == io::ErrorKind::ConnectionAborted
        || e.kind() == io::ErrorKind::ConnectionReset
}

/// Invokes client get URL, return a stream of response durations.
/// Note: Does *not* spawn new threads. Requests are concurrent not parallel.
/// Async code runs on the caller thread.
//     session: &'a hyper::Client<hyper::client::HttpConnector>
///    -> impl Stream<Item=Duration> + 'a {
#[instrument]
fn make_stream<'a>(
    session: &'a surf::Client,
    // statement: &'a hyper::Uri,
    count: usize,
) -> impl futures::Stream + 'a {
    let concurrency_limit = 5000;

    futures::stream::iter(0..count)
        .map(move |_| async move {
            //let statement = statement.clone();
            debug!("Concurrently iterating client code as future");
            let query_start = tokio::time::Instant::now();
            let mut response = session.get("/").await.expect("Surf response");
            // let (_parts, _body)  = response.unwrap().into_parts();
            let body = response.body_string().await.expect("Surf body");
            debug!("\nSTATUS:{:?}\nBODY:\n{:?}", response.status(), body);
            query_start.elapsed()
        })
        // This will run up to `concurrency_limit` futures at a time:
        .buffer_unordered(concurrency_limit)
}

impl std::fmt::Debug for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("URL")
            //.field("other_field", &self.other_field)
            .finish()
    }
}
// session: hyper::Client<hyper::client::HttpConnector>,
#[instrument]
async fn run_stream_ct(
    session: surf::Client,
    //statement: &'static URL,
    count: usize,
) {
    // Construct a local task set that can run `!Send` futures.
    let local = tokio::task::LocalSet::new();
    // Run the local task set.
    local
        .run_until(async move {
            let task = tokio::task::spawn_local(async move {
                let session = &session;
                //let statement = &statement;
                debug!("About to make stream");
                let mut stream = make_stream(&session, count);
                while let Some(duration) = stream.next().await {
                    debug!("Stream next polled.");
                }
            })
            .await
            .unwrap();
        })
        .await;
    //task.await;
}

#[instrument]
async fn capacity(count: usize) {
    debug!("About to init server");
    // Spawn a server thread setup to use the channel pattern for stopping..
    ///https://matklad.github.io/2018/03/03/stopping-a-rust-worker.html
    let mut server = Some(spawn_server());
    if let Some(ref server) = server {
        // Any String will start the server...
        server.send(Msg::Echo("Start".to_string())).unwrap();
        println!("The server WAS spawned!");
    } else {
        println!("The server was NOT spawned!");
    };

    let mut server2 = Some(spawn_server());
    if let Some(ref server2) = server2 {
        // Any String will start the server...
        server2.send(Msg::Echo("Start".to_string())).unwrap();
        println!("The server2 WAS spawned!");
    } else {
        println!("The server2 was NOT spawned!");
    };
    //init_real_server().await;
    debug!("About to init client");
    //let session = Client::new().client;
    let session: surf::Client = surf::Config::new()
        .set_http_client(http_client::h1::H1Client::new())
        .set_base_url(URL.clone())
        //.set_timeout(Some(std::time::Duration::from_secs(180)))
        .set_tcp_no_delay(true)
        .set_http_keep_alive(true)
        .set_max_connections_per_host(5000)
        .try_into()
        .unwrap();
    //let statement = &URL;
    let benchmark_start = tokio::time::Instant::now();
    debug!("Client: About to spawn blocking (Tokio)");
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        debug!("Client: About to block on (Tokio)");
        rt.block_on(async {
            let local = tokio::task::LocalSet::new();
            debug!("Client: About to run until (local set)");
            local
                .run_until(async move {
                    debug!("Client: About to spawn local (Tokio)");
                    tokio::task::spawn_local(run_stream_ct(session.clone(), count / 2))
                        .await
                        .expect("Tokio spawn local (streams for clients)");
                })
                .await;
        });
    })
    .await
    .unwrap();
    // Stop the server thread using the channel pattern...
    // https://matklad.github.io/2018/03/03/stopping-a-rust-worker.html
    drop(server.take().expect("MIO server stopped"));
    drop(server2.take().expect("MIO server2 stopped"));
    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * count as f64 / benchmark_start.elapsed().as_micros() as f64
    );
}

fn calibrate_limit(c: &mut Criterion) {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .expect("Tracing subscriber in benchmark");
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
    config = Criterion::default().with_profiler(PProfProfiler::new(3, Output::Protobuf));
    targets = calibrate_limit
}

criterion_main!(benches);
