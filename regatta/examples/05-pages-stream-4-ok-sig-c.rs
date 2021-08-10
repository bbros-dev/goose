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
    pub async fn handle_signal() {
        let sgnl = tokio::signal::ctrl_c().await;
        println!("Trapped signal clt-c to quit. Exiting!");
        std::process::exit(1)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signals_task = tokio::spawn(mymod::handle_signal());

    println!("First 10 pages:\n{:?}", get_n_pages(10).await);

    signals_task.await?;
    Ok(())
}

async fn get_n_pages(n: usize) -> Vec<Vec<usize>> {
    get_pages().take(n).collect().await
}

fn get_pages() -> impl Stream<Item = Vec<usize>> {
    stream::iter(0..).then(|i| get_page(i))
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
