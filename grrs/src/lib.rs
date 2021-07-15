// Start error-chain setup
// `error_chain!` can recurse deeply
// #![recursion_limit = "1024"]

// use anyhow::Error;

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
// #[macro_use]
// extern crate error_chain;

#[macro_use]
extern crate clap;
extern crate clap_verbosity_flag;
// #[macro_use]
// extern crate log;
use anyhow::{anyhow, bail, Context, Result};
use clap::{App, Arg};
//use error_chain::example_generated::ResultExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::ffi::OsString;
use std::io::{self};
use thiserror::Error;

use std::string::ToString;
// use strum_macros::Display;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
// mod errors {
//     error_chain! {
//         foreign_links {
//             Io(::std::io::Error);
//         }
//     }
// }

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
// pub use errors::*;

// one of the best draws of Rust is strongly typed applications.
// To make that a reality, have an intermediate step between the matching
// and the actual application code.
// This is where we extract the command line options into a struct.
// This isolates parsing logic, and the compiler helps everywhere else.
#[derive(Debug, PartialEq)]
pub struct CliArgs {
    path: String,
    pattern: String,
    version: String,
}

// This is what structopt does.
// It is done directly with clap to use the clap YAML file.
// This pattern is extended from:
// https://www.fpcomplete.com/rust/command-line-parsing-clap/
impl CliArgs {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self::new_from(std::env::args_os().into_iter())?)
    }
    pub fn new_from<I, T>(args: I) -> Result<Self, anyhow::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        // basic app CLI information
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches_from_safe(args)?;
        // Extract the path
        let path = matches
            .value_of("path")
            .expect("This can't be None, it is required.");
        let pattern = matches
            .value_of("pattern")
            .expect("This can't be None, it is required.");
        // Validate inputs
        let valid_pattern = match pattern.is_empty() {
            true => { bail!("Invalid pattern.") } ,
            false => {pattern},
        };
        Ok(CliArgs {
            path: path.to_string(),
            pattern: valid_pattern.to_string(),
            version: crate_version!().to_string(),
        })
    }
}
// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
pub fn run() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::new()?;
    // clap crate YAML configuration
    //let yaml = load_yaml!("cli.yml");
    //let matches = App::from_yaml(yaml).get_matches();
    //let path = matches.value_of("path");
    //let args = matches.args;
    // let args = Cli::from_args();
    let pb = ProgressBar::new_spinner();
    let stdout = io::stdout();
    let handle = io::BufWriter::new(stdout.lock());

    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{:?}`", &args.path))?;

    pb.enable_steady_tick(120);
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
            .template("{spinner:.white} {msg}"),
    );
    pb.set_message("Inspecting...");
    //pb.finish_with_message("Done");

    find_matches(&content, &args.pattern, handle).with_context(|| {
        format!(
            "Unable to find pattern {} in file {}",
            &args.pattern, &args.path
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_find_matches() {
        let mut output = Vec::new();
        let _result = find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut output);
        assert_eq!(output, b"lorem ipsum\n");
    }
}

/// Search for a pattern in a multi-line string.
//  Display the lines that contain it.
pub fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl std::io::Write,
) -> Result<(), anyhow::Error> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)
                .with_context(|| format!("Writing match result to stdout: {}", line))?;
        }
    }
    Ok(())
}
