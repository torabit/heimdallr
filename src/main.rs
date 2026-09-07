//! The command line. `watch` and its flags are added by the issues that define them.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::process::ExitCode;

use clap::Parser;

#[derive(Parser)]
#[command(name = "heimdallr", version, about, long_about = None)]
struct Cli {}

fn main() -> ExitCode {
    let _cli = Cli::parse();

    match heimdallr::detect() {
        Ok(mode) => {
            // The whole of stdout, so `$(heimdallr)` is a bare word.
            println!("{mode}");
            ExitCode::SUCCESS
        }
        Err(undetectable) => {
            eprintln!("error: {undetectable}");
            ExitCode::FAILURE
        }
    }
}
