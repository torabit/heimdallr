//! The command line's contract, run against the built binary.
//!
//! Neither test assumes the host has a system theme. A machine with one and a machine without
//! take different branches, and both branches are asserted.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::{Command, Output};

fn run() -> Output {
    Command::new(env!("CARGO_BIN_EXE_heimdallr"))
        .output()
        .expect("the binary under test should be runnable")
}

#[test]
fn stdout_carries_the_mode_and_nothing_else() {
    let output = run();
    let stdout = String::from_utf8(output.stdout).unwrap();

    if output.status.success() {
        assert!(
            matches!(stdout.as_str(), "dark\n" | "light\n"),
            "{stdout:?}"
        );
    } else {
        // Undetectable. Printing anything here would put it inside `$(heimdallr)`.
        assert!(stdout.is_empty(), "{stdout:?}");
    }
}

#[test]
fn an_undetectable_mode_exits_non_zero_and_says_why() {
    let output = run();
    if output.status.success() {
        return;
    }

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error: "), "{stderr:?}");
    assert!(stderr.trim().len() > "error: ".len(), "{stderr:?}");
}
