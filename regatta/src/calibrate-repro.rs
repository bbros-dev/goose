use async_stream::stream;
use futures_util::pin_mut;

async fn hello(_: Request<Body>) -> std::result::Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(HELLO)))
}

fn server_stream<'a, F: 'a>(
    session: std::sync::Arc<std::sync::Mutex<<F as futures_util::Future>::Output>>,
    count: usize,
) -> impl futures::stream::Stream<Item = F> + 'a
where
    F: futures_util::Future + futures_util::Future<Output = F> + std::fmt::Debug,
{
    stream! {
        for i in 0..count {
            let sc = session.clone();
            let s = std::sync::Arc::try_unwrap(sc).unwrap();
            let s = s.into_inner().unwrap();
            yield s.await;
        }
    }
}

async fn make_server_stream<'a, F: 'a, S: 'a>(
    session: std::sync::Arc<std::sync::Mutex<F>>,
    count: usize,
) -> impl futures_util::Future + futures_util::Stream<Item = S> + 'a
where
    F: futures_util::Future + futures_util::Stream + std::fmt::Debug,
    S: futures_util::Stream + std::iter::Iterator,
{
    let concurrency_limit = 512;
    let sc = session.clone();
    let s = server_stream(sc, count);
    pin_mut!(s); // needed for iteration
    while let Some(f) = s.next().await {
        f.buffer_unordered(concurrency_limit);
    }
}

async fn run_ct(http: hyper::server::conn::Http) {
    let http = http.clone();
    let addr = SCKTADDR.parse().unwrap();
    let listener = reuse_listener(&addr).await.expect("couldn't bind to addr");
    loop {
        let incoming = listener
            .accept()
            .await
            .map(|(stream, socket)| {
                stream.set_nodelay(true).unwrap();
                stream
            })
            .unwrap();
        let serve = http.serve_connection(incoming, service_fn(hello));

        // Run service
        tokio::task::spawn(async move {
            if let Err(http_err) = serve.await {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}

pub async fn init_server() {
    let mut threads = vec![];
    for _ in 0..(num_cpus::get() * 2) {
        let handle = tokio::runtime::Handle::current();
        let t = std::thread::spawn(move || {
            let mut http = hyper::server::conn::Http::new();
            http.http1_only(true);
            http.pipeline_flush(true);
            handle.spawn(async move {
                    run_ct(http).await;
            });
        });
        threads.push(t);
    }
    for t in threads {
        t.join().expect("Thread panicked");
    }
}

#[tokio::main]
async fn main() {
    init_server().await;
}

