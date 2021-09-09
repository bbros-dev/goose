pub mod client;
pub mod error;
pub mod handler;

use tracing::{self, debug, error, info, span, trace, warn, Instrument as _, Level, Span};
//use tracing_subscriber::{filter::EnvFilter, reload::Handle};
use tracing::instrument;
//use tracing_attributes::instrument;

use crate::calibrate::client::HttpClient;
//use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Response, Server as HyperServer};
use lazy_static::lazy_static;
//use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
//use warp::{Filter, Rejection, Reply};

type Result<T> = std::result::Result<T, Infallible>;

lazy_static! {
    static ref SERVER: tokio::sync::RwLock<Server> = tokio::sync::RwLock::new(Server::new());
    static ref SCKTADDR: String = "127.0.0.1:8888".to_string();
}
static HELLO: &[u8] = b"Hello World!";

#[instrument]
async fn hello(_: Request<Body>) -> std::result::Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(HELLO)))
}

// async fn hello(content: hyper::body::Bytes) -> std::result::Result<Response<Body>, Infallible> {
//     Ok(Response::new(Body::from(content)))
// }

// #[tokio::main]
async fn run_simple() {
    //let http_client = crate::calibrate::client::Client::new();

    //pretty_env_logger::init();

    //let bytes = hyper::body::Bytes::from_static(b"Hello World!");

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_| async {
        // Documentation: For Bytes implementations which refer to constant
        // memory (e.g. created via Bytes::from_static()) the cloning
        // implementation will be a no-op.
        //let bytes = bytes.clone();

        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        //async { Ok::<_, Infallible>(service_fn(hello)) }
        Ok::<_, Infallible>(service_fn(hello))
    });

    let addr = ([127, 0, 0, 1], 8888).into();

    let server = HyperServer::bind(&addr).serve(make_svc);

    debug!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    //Ok(())
}

// Run for HyperServer restricted to current thread
// -> futures::future::Ready<std::result::Result<i32,i32>>
#[instrument]
async fn run_ct(mut http: hyper::server::conn::Http) {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    // let make_svc = make_service_fn(|_| async {
    //     // Documentation: For Bytes implementations which refer to constant
    //     // memory (e.g. created via Bytes::from_static()) the cloning
    //     // implementation will be a no-op.
    //     //let bytes = bytes.clone();

    //     // This is the `Service` that will handle the connection.
    //     // `service_fn` is a helper to convert a function that
    //     // returns a Response into a `Service`.
    //     //async { Ok::<_, Infallible>(service_fn(hello)) }
    //     Ok::<_, Infallible>(service_fn(hello))
    // });

    //let addr = ([127, 0, 0, 1], 8888).into();

    //let mut http = hyper::server::conn::Http::new();
    //http.http1_only(true);

    // Bind to 127.0.0.1:8888
    let http = http.clone();
    let span = span!(Level::DEBUG, "run-current-thread");
    let addr = SCKTADDR.parse().unwrap();
    debug!("About to reuse address for Hyper server.");
    let listener = reuse_listener(&addr).await.expect("couldn't bind to addr");
    debug!("About to poll to accept Hyper server connection.");
    loop {
        let incoming = listener
            .accept()
            .await
            .map(|(stream, socket)| {
                stream.set_nodelay(true).unwrap();
                //socket.set_keepalive(Some(std::time::Duration::from_secs(7200))).unwrap();
                stream
            })
            .unwrap();
        let serve = http.serve_connection(incoming, service_fn(hello));

        // Run service
        debug!("About to run service on Hyper server: {:?} {:?}.", listener, addr);
        tokio::task::spawn(async move {
            if let Err(http_err) = serve.await {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
    // let server = HyperServer::bind(&addr)
    //     .executor(LocalExec)
    //     .http1_only(true)
    //     .serve(make_svc)
    //     .instrument(span);

    //.serve(|| make_service_fn(|_| Response::new(Body::from("Hello World"))))
    //.map_err(|e| eprintln!("server error: {}", e));
    //tokio::run(server);

    //let server = HyperServer::bind(&addr).executor(LocalExec).serve(make_svc);
    // Give the server a moment to start.
    // debug!("About to pause for Hyper server startup.");
    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // debug!(
    //     "Listening on thread {:?} at http://{}",
    //     std::thread::current().id(),
    //     addr
    // );

    // if let Err(e) = server.await {
    //     eprintln!("server error: {}", e);
    // }
    //futures::future::ok::<i32, i32>(0)
}

// Run for multiple HyperServer restricted to current thread.
// This closure captures as its environment the following:
// tcp: tokio::net::TcpListener
// mut http: &mut hyper::server::conn::Http
// handle: &tokio::runtime::Runtime
// statelock: tokio::sync::RwLock<Server>
//
//async fn prep_connection<F>(per_connection: F)
//futures::future::Ready<std::result::Result<bool, bool>>
//#[instrument]
async fn prep_connection<F, Fut>(
    per_connection: F,
    tcp: tokio::net::TcpListener,
    mut http: &mut hyper::server::conn::Http,
    handle: &tokio::runtime::Runtime,
    statelock: tokio::sync::RwLock<Server>,
) where
    F: Fn(tokio::net::TcpStream, &mut hyper::server::conn::Http, &tokio::runtime::Runtime) -> Fut
        + Send
        + 'static,
    Fut: std::future::Future<Output = ()>,
{
    //let span = span!(Level::DEBUG, "prepare-connection");
    debug!("Preparing connection.");
    //let state = statelock.read().await;

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_| async {
        // Documentation: For Bytes implementations which refer to constant
        // memory (e.g. created via Bytes::from_static()) the cloning
        // implementation will be a no-op.
        //let bytes = bytes.clone();

        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        //async { Ok::<_, Infallible>(service_fn(hello)) }
        Ok::<_, Infallible>(service_fn(hello))
    });

    let addr = ([127, 0, 0, 1], 8888).into();

    let server = HyperServer::bind(&addr).executor(LocalExec).serve(make_svc);
    //.instrument(span);
    // Give the server a moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    debug!(
        "Listening on thread {:?} at http://{}",
        std::thread::current().id(),
        addr
    );

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    // For every accepted connection, spawn an HTTP task
    let state = statelock.read().await;
    state.started.store(true, Ordering::Relaxed);
    drop(state);
    loop {
        let (socket, _addr) = tcp.accept().await.expect("TCP connection");
        //let _ = socket.set_nodelay(true);
        //debug!(%socket, "New connection");
        per_connection(socket, &mut http, &handle).await;
        //futures::future::ok::<bool, bool>(true);
    }
}

/// Run Hyper server using multiple threads, with a server restricted to
/// the current thread is starts in.
// futures::future::Ready<std::result::Result<bool, bool>>
//#[instrument]
pub(crate) fn prepare_servers<F, Fut>(per_connection: F)
where
    F: Fn(tokio::net::TcpStream, &mut hyper::server::conn::Http, &tokio::runtime::Runtime) -> Fut
        + Clone
        + Send
        + 'static,
    Fut: std::future::Future<Output = ()>,
{
    for _ in 0..num_cpus::get() {
        let per_connection = per_connection.clone();
        std::thread::spawn(move || async {
            debug!("Starting new std::thread for server.");
            let srv_rwlck = tokio::sync::RwLock::new(Server::new());
            server_thread(per_connection, srv_rwlck).await;
        });
    }
    //server_thread(per_connection);
}

// futures::future::Ready<std::result::Result<bool, bool>>
pub(crate) async fn server_thread<F, Fut>(per_connection: F, statelock: tokio::sync::RwLock<Server>)
where
    F: Fn(tokio::net::TcpStream, &mut hyper::server::conn::Http, &tokio::runtime::Runtime) -> Fut
        + Send
        + 'static,
    Fut: std::future::Future<Output = ()>,
{
    debug!("Setting up server thread.");
    // Setup scope that reads then release the RwLock
    let state = statelock.read().await;
    //let span = span!(Level::DEBUG, "server_thread");
    if !state.started.load(Ordering::Relaxed) {
        drop(state); // release the lock before proceeding
        let mut http = hyper::server::conn::Http::new();
        http.http1_only(true);

        // Our event loop...
        // let mut core = Core::new().expect("core");
        // let handle = core.handle();
        // Configure a current-thread only runtime (event loop)
        let handle = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread runtime");

        // Bind to 127.0.0.1:8888
        let addr = SCKTADDR.parse().unwrap();
        let tcp = reuse_listener(&addr).await.expect("couldn't bind to addr");

        // Combine `rt` with a `LocalSet`, for spawning !Send futures...
        let local = tokio::task::LocalSet::new();
        debug!("About to enter block_on prep_connection(...)");
        local.block_on(
            &handle,
            prep_connection(per_connection, tcp, &mut http, &handle, statelock),
        );
        //.instrument(span);
    }
    //core.run(server).expect("server");
}

// async fn reuse_listener(addr: &std::net::SocketAddr, handle: &Handle) -> Result<TcpListener> {
async fn reuse_listener(addr: &std::net::SocketAddr) -> Result<tokio::net::TcpListener> {
    let builder = match *addr {
        std::net::SocketAddr::V4(_) => tokio::net::TcpSocket::new_v4().expect("TCP v4"),
        std::net::SocketAddr::V6(_) => tokio::net::TcpSocket::new_v6().expect("TCP v6"),
    };
    builder.set_reuseport(true).expect("Reusable port");
    builder.set_reuseaddr(true).expect("Reusable address");
    builder.bind(*addr).expect("TCP socket");
    Ok(builder.listen(1024).expect("TCP listener"))
}

// HyperServer needs to spawn background tasks, hence
// configure an Executor that can spawn !Send futures...
#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        // This will spawn into the running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}

// Pooled connections to server (DB)... try to extend to hyper.
// https://stackoverflow.com/q/57076970
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=af120bda3f2354498f08f1d44d0a5925

// Server abstraction that guards server restarts during calibration loops.
#[derive(Debug)]
pub struct Server {
    pub started: AtomicBool,
}

impl Server {
    pub fn new() -> Server {
        Server {
            started: AtomicBool::new(false),
        }
    }

    /// Launch multiple Hyper servers. Each running in its current thread, via
    /// a LocalSet.
    #[instrument]
    pub async fn init_server(&mut self) {
        //if !self.started.load(Ordering::Relaxed) {
        debug!("Setting up server threads.");
        let mut threads = vec![];
        for _ in 0..(num_cpus::get() * 2) {
            //let threads: Vec<_> = (0..(num_cpus::get() * 2))
            //.map(|thread_id| {
            // let per_connection = per_connection.clone();
            debug!("CPU counter.");
            debug!("About to spawn Tokio task for Hyper server.");
            // tokio::task::spawn( async move {
            //     let rt = tokio::runtime::Handle::current();
            //     debug!("About to block on runtime for server (Tokio)");
            //     rt.block_on( async {
            //         let local = tokio::task::LocalSet::new();
            //         debug!("About to run until for server (local set)");
            //         local.run_until(async move {
            //             debug!("About to spawn local for server (Tokio)");
            //             tokio::task::spawn_local(run_ct()).await.expect("Tokio spawn local");
            //         }).await;
            //     });
            // }).await.unwrap();
            //start suspended
            debug!("About to spawn new std::thread for Hyper server.");
            // Set handle to be passed in to new std::thread
            let handle = tokio::runtime::Handle::current();
            let t = std::thread::spawn(move || {
                //     debug!("Starting new std::thread for server.");
                let mut http = hyper::server::conn::Http::new();
                http.http1_only(true);
                http.pipeline_flush(true);
                //     let srv_rwlck = tokio::sync::RwLock::new(Server::new());
                //     // Setup scope that reads then release the RwLock
                //     let state = srv_rwlck.read().await;
                let span = span!(Level::DEBUG, "server_thread");
                //     if !state.started.load(Ordering::Relaxed) {
                //         state.started.store(true, Ordering::Relaxed);
                //         drop(state); // release the lock before proceeding
                //                      // let mut http = hyper::server::conn::Http::new();
                //                      // http.http1_only(true);
                //                      // Our event loop...
                //                      // let mut core = Core::new().expect("core");
                //                      // let handle = core.handle();
                //         // Configure a current-thread only runtime (event loop)
                // let handle = tokio::runtime::Builder::new_current_thread()
                //         .enable_all()
                //         .build()
                //         .expect("build current-thread runtime");
                debug!("About to spawn async task from outside the Tokio runtime.");
                handle.spawn(async move {
                    //debug!("About to build new Tokio thread for Hyper server.");

                        //let local = tokio::task::LocalSet::new();
                        //debug!("About to block for server (local set)");
                        //local.run_until(run_ct()).await;
                        debug!("About to run server");
                        run_ct(http).await;

                    //futures::future::ok<i32, i32>(0);
                });
                //         // Bind to 127.0.0.1:8888
                //         // let addr = SCKTADDR.parse().unwrap();
                //         // let tcp = reuse_listener(&addr).await.expect("couldn't bind to addr");
                //         // Combine `rt` with a `LocalSet`, for spawning !Send futures...
                //    }
            });
            threads.push(t);
            //end suspended
        }
        //).collect::<Vec<_>>();
        for t in threads {
            t.join().expect("Thread panicked");
        }
        // std::thread::spawn(move || {
        //     // Configure a current-thread only runtime
        //     let rt = tokio::runtime::Builder::new_current_thread()
        //         .enable_all()
        //         .build()
        //         .expect("build current-thread runtime");
        //     // Combine `rt` with a `LocalSet`, for spawning !Send futures...
        //     let local = tokio::task::LocalSet::new();
        //     local.block_on(&rt, run_ct());
        //     // loop {
        //     //         std::thread::sleep(std::time::Duration::from_millis(100_000));
        //     //     }
        // });

        // std::thread::spawn(move || {
        //     let rt = tokio::runtime::Runtime::new().expect("runtime starts");
        //     rt.spawn( run_simple() );
        //     loop {
        //         std::thread::sleep(std::time::Duration::from_millis(100_000));
        //     }
        // });
        //tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        //self.started.store(true, Ordering::Relaxed);
        //}
    }

    //#[instrument]
    pub async fn init_server_mct(&mut self) {
        let plaintext_len = hyper::header::HeaderValue::from_static("12");
        let plaintext_ct = hyper::header::HeaderValue::from_static("text/plain");
        let server_header = hyper::header::HeaderValue::from_static("regatta");

        prepare_servers(move |socket, http, handle| async {
            //     // This closure is run for each connection...
            //     // The plaintext benchmarks use pipelined requests.
            //http.pipeline_flush(true);
            debug!("Within init_server_mct - per_connection");
            //futures::future::ok::<bool, bool>(true)
            //true
            //     // Gotta clone these to be able to move into the Service...
            //     let plaintext_len = plaintext_len.clone();
            //     let plaintext_ct = plaintext_ct.clone();
            //     let server_header = server_header.clone();

            //     // // Combine `rt` with a `LocalSet`, for spawning !Send futures...
            //     // let local = tokio::task::LocalSet::new();
            //     // local.block_on(handle, run_ct());

            //     // This is the `Service` that will handle the connection.
            //     // `service_fn_ok` is a helper to convert a function that
            //     // returns a Response into a `Service`.
            //     // let svc = service_fn_ok(move |req| {
            //     //     let (req, _body) = req.into_parts();
            //     //     // For speed, reuse the allocated header map from the request,
            //     //     // instead of allocating a new one. Because.
            //     //     let mut headers = req.headers;
            //     //     headers.clear();

            //     //     headers.insert(CONTENT_LENGTH, plaintext_len.clone());
            //     //     headers.insert(CONTENT_TYPE, plaintext_ct.clone());
            //     //     Body::from(HELLO_WORLD);

            //     //     headers.insert(SERVER, server_header.clone());

            //     //     let mut res = Response::new(body);
            //     //     *res.headers_mut() = headers;
            //     //     res
            //     // });

            //     // let make_svc = make_service_fn(|_| async {
            //     //     // Documentation: For Bytes implementations which refer to constant
            //     //     // memory (e.g. created via Bytes::from_static()) the cloning
            //     //     // implementation will be a no-op.
            //     //     //let bytes = bytes.clone();

            //     //     // This is the `Service` that will handle the connection.
            //     //     // `service_fn` is a helper to convert a function that
            //     //     // returns a Response into a `Service`.
            //     //     //async { Ok::<_, Infallible>(service_fn(hello)) }
            //     //     Ok::<_, Infallible>(service_fn(hello))
            //     // });

            //     // Spawn the `serve_connection` future into the runtime.
            //     //handle.spawn(*http.serve_connection(socket, make_svc));
        })
    }
}

#[instrument]
pub async fn init_real_server() {
    debug!("Initializing servers - multiple current thread Hyper servers");
    SERVER.write().await.init_server().await;
}
