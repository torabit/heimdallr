// Detection, `watch` and its flags are added by the issues that define them; see CLAUDE.md.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use clap::Parser;

#[derive(Parser)]
#[command(name = "heimdallr", version, about, long_about = None)]
struct Cli {}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    anyhow::bail!(
        "detection is not implemented yet; see https://github.com/torabit/heimdallr/issues"
    )
}
