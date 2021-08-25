pub mod client;
pub mod error;
pub mod handler;

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
}
static HELLO: &[u8] = b"Hello World!";

async fn hello(_: Request<Body>) -> std::result::Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(HELLO)))
}

// async fn hello(content: hyper::body::Bytes) -> std::result::Result<Response<Body>, Infallible> {
//     Ok(Response::new(Body::from(content)))
// }

// #[tokio::main]
async fn run() {

    //let http_client = crate::calibrate::client::Client::new();

    //pretty_env_logger::init();

    //let bytes = hyper::body::Bytes::from_static(b"Hello World!");

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn( |_| async {
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

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    //Ok(())
}

// fn router(
//     http_client: impl crate::calibrate::client::HttpClient,
// ) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
//     let todo = warp::path("todo");
//     let todo_routes = todo
//         .and(warp::get())
//         .and_then(handler::list_todos_handler)
//         .or(todo
//             .and(warp::post())
//             .and(with_http_client(http_client.clone()))
//             .and_then(handler::create_todo));

//     todo_routes.recover(error::handle_rejection)
// }

// Pooled connections to server (DB)... try to extend to hyper.
// https://stackoverflow.com/q/57076970
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=af120bda3f2354498f08f1d44d0a5925

// Server abstraction that guards server restarts during calibration loops.
pub struct Server {
    pub started: AtomicBool,
}

impl Server {
    pub fn new() -> Server {
        Server {
            started: AtomicBool::new(false),
        }
    }

    pub async fn init_server(&mut self) {
        if !self.started.load(Ordering::Relaxed) {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("runtime starts");
                rt.spawn( run() );
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100_000));
                }
            });
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            self.started.store(true, Ordering::Relaxed);
        }
    }

    fn reuse_listener(addr: &SocketAddr, handle: &Handle) -> io::Result<TcpListener> {
        let builder = match *addr {
            SocketAddr::V4(_) => std::TcpListener::bind("127.0.0.1:8888")?,
            SocketAddr::V6(_) => std::TcpListener::bind("[::1]:8888").unwrap()?,
        };

        #[cfg(unix)]
        {
            //use nix::sys::socket::{self, sockopt::ReusePort};
            if let Err(e) = nix::sys::socket::setsockopt(builder.as_raw_fd(), nix::sys::sockopt::ReusePort, &true) {
                eprintln!("error setting SO_REUSEPORT: {}", e);
            }
            if let Err(e) = nix::sys::socket::setsockopt(builder.as_raw_fd(), nix::sys::sockopt::ReuseAddr, &true) {
                eprintln!("error setting SO_REUSEADDR: {}", e);
            }
        }

        nix::sys::socket::listen(builder.as_raw_fd(), 1024)
            .and_then(|l| tokio::net::TcpListener::from_listener(l, addr, handle))
    }
}

pub async fn init_real_server() {
    SERVER.write().await.init_server().await;
}
