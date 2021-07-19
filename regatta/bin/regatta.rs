use anyhow::Error;
use signal_hook::consts::signal::*;
use signal_hook::low_level;
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

async fn handle_signals(signals: Signals) -> Result<(), Error> {
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
                // And actually stop ourselves.
                low_level::emulate_default_handler(SIGTSTP)?;
            }
            SIGTSTP => {
                // SIGTSTP intercept Ctrl-Z at a terminal
                println!("Received SIGTSTP");
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // SIGKILL and SIGSTOP cannot be subscribed to.
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT, SIGTSTP])?;
    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals));

    // Execute program logic
    regatta::run().await?;

    // Terminate the signal stream.
    handle.close();
    signals_task.await?;

    Ok(())
}
