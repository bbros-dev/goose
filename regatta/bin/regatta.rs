//! # Swanling
//!
//! The project provides two command line applications (`regatta`, `tender`)
//! and a Rust crate (`swanling`) for coordinated omission immune HTTP load testing
//! and benchmarking.
//!
//! Regatta reads a ["VCR cassette"] of requests, makes them and records related metrics.
//! VCR cassettes can be produced via many languages: [Erlang VCR package],
//! [Erlang/Elixir VCR package], [Julia VCR package], [NodeJS VCR packages],
//! [Python VCR pip], [R-project package], [Ruby VCR gem],
//! [Rust VCR crate], [Swift VCR packages] (DVR, SwiftVCR, Vinyl)
//!
//! Starting more than one `regatta` process creates a self organizing cluster
//! that executes the VCR cassette of HTTP requests.
//! `tender` is a console user interface to the `regatta` process(es).
//! Swanling is a library for users who wish to skip VCR cassettes and write
//! their test/benchmark in Rust.
//!
//! **Minimum supported Rust version is: 1.53.0**
//!
//! This project is hosted on [Github]. We're happy to receive pull / merge
//! requests, so please submit an issue if you've found something
//! we need to improve or have a question regarding how things work.
//!
//! ## Getting Started
//!
//! To run a VCR based HTTP test, you need only [install `regatta` and `tender`].
//!
//! To write your own Rust tests, [install Rust], then add this crate to the
//! project `Cargo.toml`.
//!
//! ## Examples and Usage
//!
//! Check out the examples folder for helpful snippets of code, as well as
//! minimal configurations that fit some common use cases.
//!
//! ```
//! ```
//!
//! ### Features
//!
//! Use these by running `cargo run --features <name of feature>`
//!
//! - **buildtime-bindgen**: Generate Rust bindings during build time.
//! - **device-test**: Enable tests that requires connections to RealSense devices.
//!
//! ## Regenerating the API Bindings
//!
//! *Non-Linux users*: The current bindings are formatted for Linux. Users on systems other than Linux must run with the
//! `buildtime-bindgen` feature to reformat the bindings. See the README in realsense-sys for more.
//!
//! *Backwards compatibility*: If you're using an older librealsense version, you may enable the `buildtime-bindgen`
//! feature to re-generate the bindings. We make no claims of backwards compatibility; good luck.
//!
//! ## Special Considerations
//!
//! - **USB Current Draw**: Many RealSense devices draw more current than a standard USB cable can provide. For example,
//!   standard USB can run 0.9 amps, while the RealSense 435i draws 2 amps. Using a USB cable that doesn't have the
//!   right current capability will interfere with the USB connection on the host, and the device will seem to
//!   disconnect. A device power cycle doesn't always remedy this, either. In many cases, the host USB hub itself will
//!   need a reset. Make sure any USB cables used are able to draw at least 2 amps. Read more on the issue
//!   [here](https://support.intelrealsense.com/hc/en-us/community/posts/360033595714-D435-USB-connection-issues).
//!
//! - **USB Bandwidth**: When a device is connected, librealsense will measure the transmission speed of data across its
//!   USB connection. USB3 speeds can handle all streams running simultaneously. USB2 speeds _cannot_; trying to set a
//!   streaming configuration that is too much for USB2 will result in a failed streaming config, and will cause the
//!   program to fail. Luckily, this information can be looked up and compensated for during runtime. See the
//!   device-specific demo examples for ways to achieve this.
//!
//! - **Supported but Ignored Stream Options**: There are a few Sensor options that are registered as "supported" by the
//!   sensor, but are actually just set to their default values on runtime. These options are listed and tested in
//!   `check_supported_but_ignored_sensor_options()` device tests. Currently,
//!   [GlobalTimeEnabled](kind::Rs2Option::GlobalTimeEnabled) on the L500 is the only setting known to suffer from this.
//!   However, the test has been written in a way that makes it easy to test more Options for this same behavior.
//!
//! ## Realsense-sys: A low-level API
//!
//! The realsense-sys crate provides C bindings generated from librealsense headers. See the [realsense-sys
//! crate](https://crates.io/crates/realsense-sys) documentation for more information.
//!
//! ## Design Philosophy
//!
//! There is a preference for convention over configuration.
//! See the [architecture](book::architecture) document for detail about the
//! peer-to-peer and Raft configuration. As well as areas requiring refinement.
//!
//! [install `regatta` and `tender`]: https://swanling.io/install
//! [install Rust]: https://www.rust-lang.org/tools/install
//! [Github]: https://github.com/BegleyBrothers/swanling
//! [Erlang/Elixir VCR package]: https://hex.pm/packages/exvcr
//! [Julia VCR package]: https://github.com/JuliaTesting/BrokenRecord.jl
//! [NodeJS VCR packages]: https://www.npmjs.com/search?q=vcr
//! [Python VCR pip]: https://pypi.org/project/vcrpy/
//! [R-project package]: https://cran.r-project.org/web/packages/vcr/index.html
//! [Rust VCR crate]: https://crates.io/crates/surf-vcr
//! [Ruby VCR gem]: https://rubygems.org/gems/vcr/versions/3.0.1
//! [Swift VCR packages]: https://swiftpackageindex.com/
//! ["VCR cassette"]: https://github.com/vcr/vcr
//!
use anyhow::Error;
use rand::distributions::{Distribution, Uniform};
use signal_hook::consts::signal::*;
use signal_hook::low_level;
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

async fn handle_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            // SIGWINCH get notified when the terminal resizes
            // SIGCHLD get notified when a child process exits
            SIGHUP => {
                // SIGHUP do something when the terminal exits.
                //  - Reload configuration
                //  - Reopen the log file
                println!("Received SIGHUP");
            }
            SIGTERM | SIGINT | SIGQUIT => {
                // Shutdown the system;
                // SIGINT intercept Ctrl-C at a terminal
                // SIGQUIT intercept Ctrl-\ at a terminal
                // SIGTERM intercept the "kill" command's default signal)
                println!("Received SIGTERM SIGINT or SIGQUIT");
                // Actually stop.
                std::process::exit(1);
            }
            SIGTSTP => {
                // SIGTSTP intercept Ctrl-Z at a terminal
                println!("Received SIGTSTP");
            }
            _ => unreachable!(),
        }
    }
    //Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // SIGKILL and SIGSTOP cannot be subscribed to.
    let signals = signal_hook_tokio::Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT, SIGTSTP])?;
    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals));

    // Execute program logic
    //let millis = Uniform::from(5_000..6_000).sample(&mut rand::thread_rng());
    //tokio::time::sleep(std::time::Duration::from_millis(millis)).await;
    regatta::run().await?;

    // Terminate the signal stream.
    handle.close();
    signals_task.await?;

    Ok(())
}
