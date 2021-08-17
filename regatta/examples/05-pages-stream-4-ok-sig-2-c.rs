//Add to Cargo.toml
//ctrlc = { version = "3.1", features = ["termination"] }
use async_stream::stream;
use futures::{stream, Stream, StreamExt};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::time::Duration;
use tokio::time::{sleep, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub mod cli {

    /// Listens for then tracks the CLI shutdown signal.
    ///
    /// Shutdown is signalled by the user (ctl-c) and managed using a
    /// `tokio::sync::oneshot::channel`.
    /// Only a single value is ever sent on this channel.
    /// Once a value has been sent via the oneshot channel, the CLI application
    /// should shutdown.
    ///
    /// The `Shutdown` struct listens for the shutdown signal and tracks
    /// the signal state.
    /// Callers are passed this struct and may query the shutdown signal state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct Signal {
    ///     shutdown: Shutdown,
    /// }
    /// let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    ///
    /// let mut signal = Signal {
    ///     shutdown: Shutdown.new(shutdown_rx)
    /// }
    /// println!("Do we shutdown now? {:?}", signal.shutdown.is_now);
    /// assert!(!signal.shutdown.is_now);
    /// ```
    #[derive(Debug)]
    pub(crate) struct Shutdown<'a> {
        /// `true` if the shutdown signal has been received
        pub is_now: bool,

        /// The receive (RX) half of the channel used to listen for shutdown.
        listen: &'a mut tokio::sync::oneshot::Receiver<()>,
    }

    impl<'a> Shutdown<'a> {
        /// Create a new `Shutdown` backed by the given `broadcast::Receiver`.
        /// We'll borrow the Receiver when creating a new shutdown
        pub(crate) fn new(listen: &'a mut tokio::sync::oneshot::Receiver<()>) -> Shutdown {
            Shutdown {
                is_now: false,
                listen,
            }
        }

        /// Check for a shutdown notice. This uses a synchronized channel from
        /// Tokio waiting is unnecessary.
        pub(crate) async fn check(&mut self) {
            println!("Checking shutdown signal...");
            // If the shutdown signal has already been received, then return
            // immediately.
            if self.is_now {
                println!("... signal already received.");
                return;
            }

            // Cannot receive a "lag error" as only one value is ever sent.
            // Can receive the signal after the TX end has closed, if the
            // signal was sent before the TX end was closed.
            // NOTE:
            //      We consider the `try_recv` call as non-blocking.
            //      It is not asynchronous (cannot use `try_recv().await`).
            match self.listen.try_recv() {
                Ok(_) => {
                    // Remember that the signal has been received.
                    self.is_now = true;
                    // Drop the receiver - forcing the sender to close.
                    println!("User-sent abort/quit/exit signal detected.")
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    // A message can be sent before the sender `closed()`,
                    // and still be receivable.
                    match self.listen.try_recv() {
                        Ok(_) => {
                            // Remember that the signal has been received.
                            self.is_now = true;
                            // Drop the receiver - forcing the sender to close.
                            println!("User-sent abort/quit/exit signal detected.")
                        }
                        Err(_) => {
                            println!(
                                "Sender (tx) is closed. No user-sent shutdown signal detected."
                            );
                        }
                    };
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                    println!("Sender (tx) is open. No user-sent shutdown signal detected.")
                } // This is unreachable.
            };
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn it_works() {
            assert_eq!(2 + 2, 4);
        }
    }

    // This is the simplest signal handling in tokio.
    // We don't pass anything between the thread running `main` and the thread
    // `tokio::spawn`ed to run this function.
    pub async fn handle_signal(mut some_tx: Option<tokio::sync::oneshot::Sender<()>>) {
        tokio::signal::ctrl_c()
            .await
            .expect("Handle ctl-c (Tokio built in)");

        println!("Trapped signal clt-c to quit.");

        if let Some(tx) = some_tx.take() {
            match tx.send(()) {
                Ok(()) => println!("The message may be received."),
                Err(e) => println!("The message will never be received: {:?}", e),
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

struct Signal<'a> {
    shutdown: cli::Shutdown<'a>
}

#[tokio::main]
async fn real_main() -> i32 {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    // Wrap the Sender in Option to allow taking it by value, because
    // they consume themselves after one use
    let mut some_shutdown_tx = Some(shutdown_tx);

    // The Receiver does not consume itself, so no need to treat as an `Option`.
    let mut signal = Signal {
        shutdown: cli::Shutdown::new(&mut shutdown_rx),
    };

    let signals_task = tokio::spawn(cli::handle_signal(some_shutdown_tx));

    println!("First 10 pages:\n{:?}", get_n_pages(10, &mut signal).await);

    // From point on the user, *likely*, has not sent a signal.
    // Anyway, we are now on the shutdown glide-path.
    // A `rx.close()` prevents the `tx` side from sending.
    shutdown_rx.close();
    println!("If a User-signal has been sent: The `signal_handler` will shortly call `tx.closed().await`");
    // However, a `tx.send()` a 'just' before or during `rx.close()` was possible...
    let msg = match shutdown_rx.try_recv() {
        Ok(_) => {
            println!("User-sent abort/quit/exit signal detected.")
        }
        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
            println!("Sender (tx) is closed. No user-sent shutdown signal detected.")
        }
        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
            println!("Sender (tx) is open. No user-sent shutdown signal detected.")
        }
        Err(_) => {
            println!("Shutdown sender (tx) is dropped without sending.");
        }
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

async fn get_n_pages<'a>(n: usize, signal: &'a mut Signal<'a>) -> Vec<Vec<usize>> {
    signal.shutdown.check().await;
    if signal.shutdown.is_now {
        println!("Shutdown now from get_n_pages(...)");
        return vec![vec![1]];
    }
    get_pages(signal).take(n).collect().await
}

fn get_pages<'a>(mut signal: &'a mut Signal<'a>) -> impl Stream<Item = Vec<usize>> + 'a
{
    // The need for lifetimes means we need to switch to async-streams to give
    // the compiler the required insight into what is happening:
    // See: https://users.rust-lang.org/t/lifetimes-adding-a-parameter-inside-a-3rd-party-closure/63542/3?u=taqtiqa-mark
    // Note: As of 1.53 labels on blocks are an unstable feature.
    //stream::iter(0..).then(move |i| get_page(i, &mut signal) )
    stream! {
        for i in 0.. {
           yield get_page(i, &mut signal).await;
        }
    }
}

async fn get_page<'a, 'b: 'a>(i: usize, signal: &'a mut Signal<'b>) -> Vec<usize> {
    signal.shutdown.check().await;
    if signal.shutdown.is_now {
        println!("Shutdown now from get_page(...)");
        return vec![0];
    }
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
