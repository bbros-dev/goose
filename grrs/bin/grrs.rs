// use mockall::*;
// use mockall::predicate::*;
// use mockall_double::double;

// use structopt::StructOpt;

// #[derive(Clone, Debug, StructOpt)]
// struct Cli {
//     #[structopt(flatten)]
//     verbose: clap_verbosity_flag::Verbosity,
//     /// The pattern to look for
//     pattern: String,
//     /// The path to the file to read
//     #[structopt(parse(from_os_str))]
//     path: std::path::PathBuf,
// }
 use anyhow::Error;

fn main() -> Result<(), Error> {
    // env_logger::init();

    // let args = Cli::from_args();
    // let pb = ProgressBar::new_spinner();
    // let stdout = io::stdout();
    // let handle = io::BufWriter::new(stdout.lock());

    // let content = std::fs::read_to_string(&args.path)
    //     .with_context(|| format!("could not read file `{:?}`", &args.path))?;

    // pb.enable_steady_tick(120);
    // pb.set_style(
    //     ProgressStyle::default_spinner()
    //         // For more spinners check out the cli-spinners project:
    //         // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
    //         .tick_strings(&[
    //             "▹▹▹▹▹",
    //             "▸▹▹▹▹",
    //             "▹▸▹▹▹",
    //             "▹▹▸▹▹",
    //             "▹▹▹▸▹",
    //             "▹▹▹▹▸",
    //             "▪▪▪▪▪",
    //         ])
    //         .template("{spinner:.white} {msg}"),
    // );
    // pb.set_message("Inspecting...");

    // grrs::find_matches(&content, &args.pattern, handle)?;

    // pb.finish_with_message("Done");
    grrs::run()
}
