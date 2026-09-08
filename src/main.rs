//! The command line.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "heimdallr", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run a command now, and again whenever the system theme changes.
    Watch {
        /// The command, handed to `sh -c` with the mode as `$1`.
        #[arg(long, value_name = "COMMAND")]
        on_change: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        None => print_mode(),
        Some(Command::Watch { on_change }) => watch(&on_change),
    }
}

fn print_mode() -> ExitCode {
    match heimdallr::detect() {
        Ok(mode) => {
            // The whole of stdout, so `$(heimdallr)` is a bare word.
            println!("{mode}");
            ExitCode::SUCCESS
        }
        Err(undetectable) => fail(&undetectable),
    }
}

fn watch(on_change: &str) -> ExitCode {
    // Only returns when there is nothing left to watch; a failing command is reported here and
    // stepped over.
    match heimdallr::watch::run(on_change, |failure| eprintln!("error: {failure}")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
    }
}

fn fail(error: &dyn std::error::Error) -> ExitCode {
    eprintln!("error: {error}");
    ExitCode::FAILURE
}
