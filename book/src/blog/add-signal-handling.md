# Add Signal Handling: A detour into the Rust error database

This post builds on and emulates the call-response style post by Guillaume
 Endignoux, [Asynchronous streams in Rust (part 1)].

In that post Guillaume iterates through some of the 'dead-ends' a newish user
could reasonably expect to traverse while working on asynchronous code in Rust.
What was informative was the explanation for why some approaches worked and
others did not.
I expect to return to that post from time to time, and this post is in that
spirit - a note to future self.

Here I extend Guillaume's example by adding signal handling to one of his
examples.
In case you haven't guessed already, I decided to take
this approach in order to understand why some non-trivial code was not working.
---
**NOTE**
    The full code examples are available on [this GitHub repository].
    You can run each example (or obtain the example compilation error) with `cargo run --example <example name>`.
    At the time of writing, I’ve tested them with Rust version 1.53.0.

    ```bash
    $ cargo --version
    cargo 1.53.0 (4369396ce 2021-04-27)
    ```

---

The dependencies in `Cargo.toml` are

```toml
[package]
name = "async-examples"
version = "0.1.0"
authors = ["G. Endignoux <ggendx@gmail.com>", "Mark Van de Vyver <mark@taqtiqa.com>"]
edition = "2018"

[dependencies]
futures = "0.3.13"
lazy_static = "1.4.0"
rand = "0.8.3"
tokio = { version = "1.4.0", features = ["macros", "rt-multi-thread", "time"] }

# To plot the results
plotters = "0.3.0"
regex = "1"
```

## Interrupting Async Hello World

Our first step is to add the ability to kill the CLI using `ctl+c`.

### A Puzzling Error

However, I'll start by pointing to a puzzling error you can encounter quite easily.
This is one of those face-palm errors you make while changing gears as you shift
between languages and projects.
Fortunately, the fix is trivial, and we get an insight into the state of play
in Rust compiler error reporting.

This is not a "How we wrestled the borrow checker, and won".  Rather the issue
is much simpler, and along the way we tease out a subtle issue in the compiler
error reporting, and a small documentation gap in ["The Book"].

However, all rust versions from `1.53.0` through to
`1.56.0-nightly (5ad7389bd 2021-08-06)` provide an error message
that is unhelpful, and it is useful to bear in mind what this trait bounds
error could be obscuring.

Start with the following extension of [01-pages-hello-ok.rs], where we add some
basic signal handling logic:

```rust
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

Running this, `cargo run --example=01-pages-hello-ok`, we encounter our first
error message that is pretty unhelpful:

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
`fuse` on a type that doesn't implement it.

That is not a useful error message.  And when we see what the source of the
error is, we'll face palm, then release a string of rated statements asking why
the compiler didn't just say that.

Yet some part of the Rust story turns on [compile time error messages] - how great they are.
Why? Given that sort of obscure feedback is your early experience.

To see why, make two subtle changes:  Specifically, place a module around
`handle_signals` and add this `use` statement `use futures::stream::StreamExt;`:

```rust
// add this
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

//add module
pub mod mymod {
    // make public
    pub async fn handle_signals(signals: signal_hook_tokio::Signals) {
        let mut signals = signals.fuse();
        while let Some(signal) = signals.next().await {
            match signal {
                signal_hook::consts::SIGTERM | signal_hook::consts::SIGINT | signal_hook::consts::SIGQUIT => {
                    // Lets get out of here...
                    println!("Trapped signal to quit. Exiting");
                    std::process::exit(1);
                }
                _ => unreachable!(),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let signals = signal_hook_tokio::Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    // remember the module
    let signals_task = tokio::spawn(mymod::handle_signals(signals));

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

    tokio::time::sleep(Duration::from_millis(millis)).await;
    println!(
        "[{}] # get_page({}) completed",
        START_TIME.elapsed().as_millis(),
        i
    );

    (10 * i..10 * (i + 1)).collect()
}
```

Recompile, `cargo run --example=01-pages-hello-ok`, and you'll see the same error.
So, instead, upgrade to the forthcoming 2021 edition:

```bash
rustup update nightly
cargo +nightly fix --edition
# Edit Cargo.toml
# 1. place cargo-features = ["edition2021"] at the top (above [package])
# 2. change the edition field to say edition = "2021"
cargo +nightly check
```

If that all went smoothly, use the nightly to build the updated example:

```rust
rustup default nightly-x86_64-unknown-linux-gnu
cargo run --example=01-pages-hello-ok-sig-c
```

You should now see the opening scenes of a (geek) love story scroll down your screen:

```rust
error[E0599]: the method `fuse` exists for struct `signal_hook_tokio::SignalsInfo`, but its trait bounds were no
t satisfied
  --> regatta/examples/01-pages-hello-ok-int-c.rs:19:35
   |
19 |         let mut signals = signals.fuse();
   |                                   ^^^^ method cannot be called on `signal_hook_tokio::SignalsInfo` due to u
nsatisfied trait bounds
   |
  ::: /home/hedge/.cargo/registry/src/github.com-1ecc6299db9ec823/signal-hook-tokio-0.3.0/src/lib.rs:92:1
   |
92 | pub struct SignalsInfo<E: Exfiltrator = SignalOnly>(OwningSignalIterator<UnixStream, E>);
   | ----------------------------------------------------------------------------------------- doesn't satisfy `
signal_hook_tokio::SignalsInfo: Iterator`
   |
   = note: the following trait bounds were not satisfied:
           `signal_hook_tokio::SignalsInfo: Iterator`
           which is required by `&mut signal_hook_tokio::SignalsInfo: Iterator`
   = help: items from traits can only be used if the trait is in scope
   = note: the following trait is implemented but not in scope; perhaps add a `use` for it:
           `use futures::StreamExt;`
```

Of course,, if you are following at home you'll be mildly irritated.

1. The `use` statement we added would have fixed the original issue, and there
   is no way of knowing from the feedback that was wrong.
2. We had to wrap things in modules in order to tease useful feedback out of
   the compiler.
3. The Rust book does not cover the pattern where `use` is nested within `mod`,
   which we deployed to tease a useful error message from the compiler.

In my view that is a reasonable response.  Given the relevant crate is present in
`Cargo.toml`, most development teams now would expect some sort of feedback
along the lines we eventually teased out of the compiler.

If you want to follow the fate of this issue you can in [rust-lang issue #87857].
Of course we are not the first to hit variations of this issue, and you can
learn some important things (all positive in my view) about the Rust
development approach and priorities by reviewing [issue #36513] and [issue #40375].

## Conclusion

Rust has a compelling performance and safety story.
However, it is still early days.
As we saw variations of this issue have been around since 2015 and 2016,
so things can move slowly in Rust-land.

However, in my view, it is worth the wait.  I've worked on distributed computing
problems on and off since the 1990's (Condor, MPI, home-made, etc).
While it is early days, the concurrent/parallel approach in Rust feels like
the right level of abstraction, and the right abstractions.

[Asynchronous streams in Rust (part 1)]: https://gendignoux.com/blog/2021/04/01/rust-async-streams-futures-part1.html
[compile time error messages]: https://stackoverflow.blog/2020/06/05/why-the-developers-who-use-rust-love-it-so-much/
[issue #36513]: https://github.com/rust-lang/rust/issues/36513
[issue #40375]: https://github.com/rust-lang/rust/issues/40375
[rust-lang issue #87857]: https://github.com/rust-lang/rust/issues/87857
["The Book"]: https://doc.rust-lang.org/book
[this GitHub repository]: https://github.com/taqtiqa-mark/rust-async-examples
[01-pages-hello-ok.rs]: https://github.com/gendx/rust-async-examples/blob/main/examples/01-pages-hello-ok.rs