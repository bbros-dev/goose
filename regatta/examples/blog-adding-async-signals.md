# Adding signals

This post emulates a call-response style post by Guillaume Endignoux, [Asynchronous streams in Rust (part 1)]. In that post Guillaume iterates through some of the 'dead-ends' you
could reasonably expect to traverse while working on asynchronous code in Rust.
What was informative was the explanations of why some approaches worked and others did not.
I expect to return to that post from time to time, and this post is in that
spirit - a note to future self.

Here I extend Guillaume's examples in two ways. First by adding signal handling and then
progress indicators.  In case you haven't guessed already, I decided to take
this approach in order to understand why some non-trivial code was not working.

NOTE
    The full code examples are available on [this GitHub repository].
    You can run each example (or obtain a compilation error) with `cargo run --example <example name>`.
    At the time of writing, I’ve tested them with Rust version 1.53.0.

    $ cargo --version
    cargo 1.53.0 (4369396ce 2021-04-27)

The dependencies in `Cargo.toml` are

    [dependencies]
    ....
    signal-hook = "0.3.9"
    signal-hook-tokio = { version = "0.3.0", features = ["futures-v0_3"]}

## Quitting async Hello World

Our first step is to add the first and last three lines to our start point,
along with the function `handle_signals`.  We also need to pay attention to the
`use ...` statements:

```rust
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::io::Error;
use std::time::Duration;
use tokio::time::{sleep, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

async fn handle_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                // Lets get out of here...
                std::process::exit(1);
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_signals(signals));

    let page = get_page(42).await;
    println!("Page #42: {:?}", page);

    // Terminate the signal stream.
    handle.close();
    signals_task.await?;
    Ok(())
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
```

Running this, `cargo run --example=01-pages-hello-ok-int-c` we encounter our first error:

```text
error[E0599]: the method `fuse` exists for struct `signal_hook_tokio::SignalsInfo`, but its trait bounds were not satisfied
  --> regatta/examples/01-pages-hello-ok-int-c.rs:14:31
   |
14 |     let mut signals = signals.fuse();
   |                               ^^^^ method cannot be called on `signal_hook_tokio::SignalsInfo` due to unsatisfied trait bounds
   |
...
   |
92 | pub struct SignalsInfo<E: Exfiltrator = SignalOnly>(OwningSignalIterator<UnixStream, E>);
   | ----------------------------------------------------------------------------------------- doesn't satisfy `signal_hook_tokio::SignalsInfo: Iterator`
   |
   = note: the following trait bounds were not satisfied:
           `signal_hook_tokio::SignalsInfo: Iterator`
           which is required by `&mut signal_hook_tokio::SignalsInfo: Iterator`

error: aborting due to previous error

For more information about this error, try `rustc --explain E0599`.
```

There is no help instruction and the `rustc --explain E0599` suggests we've called
`fuse` on a type that doesn't implement it.  However the first line of the error
is more suggestive of what may have gone wrong - `... its trait bounds were not satisfied`.

[Asynchronous streams in Rust (part 1)]: https://gendignoux.com/blog/2021/04/01/rust-async-streams-futures-part1.html
[this GitHub repository]: https://github.com/taqtiqa-mark/rust-async-examples
