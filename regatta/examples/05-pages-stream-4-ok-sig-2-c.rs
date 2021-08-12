use futures::{stream::{Stream, StreamExt}};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::signal::unix::{signal, Signal, SignalKind};
use tokio::time::{sleep, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[derive(Debug)]
pub struct SignalStream {
    signal: Signal,
}

impl SignalStream {
    pub fn new(kind: SignalKind) -> Self {
        Self {
            signal: signal(kind).unwrap(),
        }
    }
}

impl Stream for SignalStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.signal.poll_recv(cx)
    }
}

pub mod mymod {

    use futures::StreamExt;

    // This can handle all tokio signals.
    // We don't pass anything between the thread running `main` and the thread
    // `tokio::spawn`ed to run this function.
    pub async fn handle_signals(
        mut interrupts: futures::stream::Fuse<super::SignalStream>,
        mut terminates: futures::stream::Fuse<super::SignalStream>,
    ) {
        loop {
            futures::select! {
                s = interrupts.next() => {
                    println!("Trapped signal to quit. {:?}. Exiting!", s);
                    std::process::exit(1)
                },
                s = terminates.next() => {
                    println!("Trapped signal to quit. {:?}. Exiting!", s);
                    std::process::exit(1)
                },
                complete => break,
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interrupts = SignalStream::new(SignalKind::interrupt()).fuse();
    let terminates = SignalStream::new(SignalKind::terminate()).fuse();

    let signals_task = tokio::spawn(mymod::handle_signals(interrupts, terminates));

    println!("First 10 pages:\n{:?}", get_n_pages(10).await);

    signals_task.await?;
    Ok(())
}

async fn get_n_pages(n: usize) -> Vec<Vec<usize>> {
    get_pages().take(n).collect().await
}

fn get_pages() -> impl Stream<Item = Vec<usize>> {
    futures::stream::iter(0..).then(get_page)
}

async fn get_page(i: usize) -> Vec<usize> {
    let millis = Uniform::from(5_000..6_000).sample(&mut rand::thread_rng());
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
