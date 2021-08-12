//Add to Cargo.toml
//ctrlc = { version = "3.1", features = ["termination"] }
use futures::{stream, Stream, StreamExt};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::time::Duration;
use tokio::time::{sleep, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub mod mymod {

    // This is the simplest signal handling in tokio.
    // We don't pass anything between the thread running `main` and the thread
    // `tokio::spawn`ed to run this function.
    pub async fn handle_signal(mut some_tx: Option<tokio::sync::oneshot::Sender<String>>) {
        //let tx = some_tx;
        tokio::signal::ctrl_c().await.expect("Handle ctl-c (Tokio built in)");
        println!("Trapped signal clt-c to quit.");
        //let Some(tx) = some_tx.take();
        if let Some(tx) = some_tx.take() {
            match tx.send("Gracefully".to_string()){
                Ok(()) => println!("The message may be received."),
                Err(e) => println!("The message will never be received: {:?}",e),
            }
        }
        if let Some(mut tx) = some_tx {
            // Wait for the associated `rx` handle to be `rx.close()` or`drop(rx)`ed
            tx.closed().await;
            println!("The receiver (rx) is closed or dropped.");
        }
        println!("Exiting!");
        //std::process::exit(1);
    }
}

// tokio::spawn is conceptually like thread::spawn.
// Specifically, you cannot externally force tasks to shutdown.
// There is an elegant workaround:
// 1. Wrap the 'working' future being spawned with `futures::future::select`
// 2. Add a 'signalling' future to the select. This future must accept external signal.
// 3. If the 'signalling' future completes, the 'working' future will be dropped cancel all its pending tasks.

// Shutdown examples:
//
//
//
//
fn main() {
    std::process::exit(real_main());
}

#[tokio::main]
async fn real_main() -> i32 {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    let mut some_shutdown_tx = Some(shutdown_tx);
    let signals_task = tokio::spawn(mymod::handle_signal( some_shutdown_tx));

    println!("First 10 pages:\n{:?}", get_n_pages(10).await);

    // From point on the user, *likely*, has not sent a signal.
    // Anyway, we are now on the shutdown glide-path.
    // A `rx.close()` prevents the `tx` side from sending.
    shutdown_rx.close();
    println!("If a User-signal has been sent: The `signal_handler` will shortly call `tx.closed().await`");
    // However, a `tx.send()` a 'just' before or during `rx.close()` was possible...
    let msg = match shutdown_rx.try_recv() {
        Ok(_) => {
            println!("User-sent abort/quit/exit signal detected.")
        },
        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
            println!("Sender (tx) is closed. No user-sent abort/quit/exit signal detected.")
        },
        Err(_) => {
            println!("Shutdown sender (tx) is dropped without sending.");
        },
        _ => unreachable!(),
    };
    // `drop(tx)` with or without sending, is achieved by setting the
    // `Option` to `None`
    some_shutdown_tx = None;
    // As the `Sender` handle held by the listener has been dropped above,
    // the "oneshot" channel will close and `recv()` will return `None`.
    //let _ = shutdown_rx.await;
    drop(signals_task);
    0
}

async fn get_n_pages(n: usize) -> Vec<Vec<usize>> {
    get_pages().take(n).collect().await
}

fn get_pages() -> impl Stream<Item = Vec<usize>> {
    stream::iter(0..).then(|i| get_page(i))
}

async fn get_page(i: usize) -> Vec<usize> {
    let millis = Uniform::from(1_000..2_000).sample(&mut rand::thread_rng());
    println!(
        "[{}] # get_page({}) will complete in {} ms on {:?}",
        START_TIME.elapsed().as_millis(),
        i,
        millis,
        std::thread::current().id()
    );

    sleep(Duration::from_millis(millis)).await;
    println!(
        "[{}] # get_page({}) completed",
        START_TIME.elapsed().as_millis(),
        i
    );

    (10 * i..10 * (i + 1)).collect()
}
