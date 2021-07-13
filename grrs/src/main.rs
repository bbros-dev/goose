// use mockall::*;
use mockall::predicate::*;
// use mockall_double::double;

extern crate clap_verbosity_flag;
#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self};
use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Clone, Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::from_args();
    let pb = ProgressBar::new_spinner();
    let stdout = io::stdout(); // get the global stdout. Acquire a lock on it.
    let handle = io::BufWriter::new(stdout.lock()); // optional: wrap that handle in a buffer

    // let content = std::fs::read_to_string(&args.path)?;
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

    // let result = std::fs::read_to_string(&args.path)?;
    // let content = match result {
    //     Ok(content) => { content },
    //     Err(error) => { return Err(error.into()); }
    // };
    // println!("file content: {}", content);
    // writeln!(handle, "file content: {}", content); // add `?` if you care about errors here
    // let pb = indicatif::ProgressBar::new();
    find_matches(&content, &args.pattern, handle)?;
    pb.finish_with_message("Done");
    Ok(())
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

fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl std::io::Write,
) {
    for line in content.lines() {
        if line.contains(pattern) {
            info!("Match found");
            writeln!(writer, "{}", line)?;
            //warn!("oops, nothing implemented!");
        }
    }
}
