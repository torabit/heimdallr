//! Running a command now, and again whenever the system theme changes.
//!
//! The watcher keeps no record of which theme is applied. The command owns that, and a second
//! record of it would drift. Being called twice with the same mode is harmless, so the command
//! runs once at start rather than waiting for a change that may already have happened while the
//! machine was asleep.

use std::process::{Command, ExitStatus};
use std::time::Duration;

use thiserror::Error;

use crate::{Mode, Undetectable, wsl};

/// The shell the command is handed to, so that `&&` and the rest of it work.
const SHELL: &str = "sh";

/// What `$0` is inside the command, so a shell diagnostic names something recognisable.
const ARGV0: &str = "heimdallr";

/// Why the watcher could not start, or could not go on.
#[derive(Debug, Error)]
pub enum Error {
    /// There is nothing to watch on a host with no system theme.
    #[error("nothing to watch: {0}")]
    Undetectable(#[from] Undetectable),
    #[error("could not watch for theme changes: {0}")]
    Backend(#[from] dark_light::Error),
    #[error("the theme watcher stopped")]
    Stopped,
}

/// Why one run of the command did not succeed.
///
/// Reported and stepped over. A command that fails once is not a reason to stop watching.
#[derive(Debug, Error)]
pub enum Failure {
    #[error("could not run `{SHELL}`: {0}")]
    Unstartable(#[from] std::io::Error),
    #[error("the command exited with {status}")]
    Exited { status: ExitStatus },
    /// A poll that answered nothing. Transient far more often than not, and dying would leave the
    /// theme unapplied until something restarts us into the same failure.
    #[error("could not re-read the Windows theme: {0}")]
    Unreadable(#[from] wsl::Error),
}

/// Runs `on_change` with the current mode, then again on every change.
///
/// `interval` is how often the Windows registry is re-read under WSL, and is not consulted
/// anywhere else: every other platform blocks until the system tells it something changed.
///
/// Only returns when the mode cannot be read at all, or when the watcher stops. Every failure of
/// `on_change` itself goes to `report` and the watcher carries on.
///
/// # Errors
///
/// Returns [`Error::Undetectable`] before anything is watched when the host has no system theme,
/// and [`Error::Backend`] when the platform watcher cannot be subscribed to.
pub fn run(
    on_change: &str,
    interval: Duration,
    mut report: impl FnMut(&Failure),
) -> Result<(), Error> {
    let mut current = crate::detect()?;

    // Once at start: nothing has applied the theme if the machine was off across the transition.
    fire(on_change, current, &mut report);

    if wsl::active() {
        poll(on_change, interval, current, &mut report);
    }

    let watcher = dark_light::subscribe()?;
    for change in watcher.iter() {
        // Unspecified is not a mode, and not a reason to run anything.
        let Ok(next) = Mode::try_from(change) else {
            continue;
        };
        if next == current {
            continue;
        }
        current = next;
        fire(on_change, current, &mut report);
    }

    Err(Error::Stopped)
}

/// Re-reads the Windows theme every `interval` and runs the command when the answer changes.
///
/// `reg.exe` offers no notification, so WSL is the one platform that polls. Every other backend
/// blocks: Windows on `RegNotifyChangeKeyValue`, Linux on a D-Bus signal, macOS on `dark_light`'s
/// own loop.
fn poll(
    on_change: &str,
    interval: Duration,
    mut current: Mode,
    report: &mut impl FnMut(&Failure),
) -> ! {
    loop {
        std::thread::sleep(interval);
        match tick(current, wsl::detect()) {
            Tick::Unchanged => {}
            Tick::Changed(next) => {
                current = next;
                fire(on_change, current, report);
            }
            Tick::Unreadable(why) => report(&Failure::from(why)),
        }
    }
}

/// What one poll means.
///
/// Split out from [`poll`] so that "a change runs the command once, not on every tick" is
/// decidable without waiting on a clock.
#[derive(Debug)]
enum Tick {
    Unchanged,
    Changed(Mode),
    Unreadable(wsl::Error),
}

fn tick(current: Mode, reading: Result<Mode, wsl::Error>) -> Tick {
    match reading {
        Ok(next) if next == current => Tick::Unchanged,
        Ok(next) => Tick::Changed(next),
        Err(why) => Tick::Unreadable(why),
    }
}

/// Runs the command once, reporting rather than propagating whatever it does.
fn fire(on_change: &str, mode: Mode, report: &mut impl FnMut(&Failure)) {
    match command(on_change, mode).status() {
        Ok(status) if status.success() => {}
        Ok(status) => report(&Failure::Exited { status }),
        Err(source) => report(&Failure::Unstartable(source)),
    }
}

/// The shell invocation for one run: `sh -c <on_change> heimdallr <mode>`, which is what puts the
/// mode in `$1`.
fn command(on_change: &str, mode: Mode) -> Command {
    let mut command = Command::new(SHELL);
    command
        .arg("-c")
        .arg(on_change)
        .arg(ARGV0)
        .arg(mode.to_string());
    command
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    fn argv(on_change: &str, mode: Mode) -> Vec<String> {
        command(on_change, mode)
            .get_args()
            .map(OsStr::to_string_lossy)
            .map(std::borrow::Cow::into_owned)
            .collect()
    }

    #[test]
    fn the_mode_arrives_as_the_first_positional_argument() {
        assert_eq!(
            argv("echo \"$1\"", Mode::Dark),
            ["-c", "echo \"$1\"", ARGV0, "dark"]
        );
    }

    #[test]
    fn the_command_goes_to_a_shell_unsplit() {
        let command = command("a && b", Mode::Light);
        assert_eq!(command.get_program(), OsStr::new(SHELL));
        assert_eq!(
            command.get_args().nth(1),
            Some(OsStr::new("a && b")),
            "the command must reach the shell in one piece"
        );
    }

    #[test]
    fn a_poll_that_answers_the_same_mode_is_not_a_change() {
        assert!(matches!(tick(Mode::Dark, Ok(Mode::Dark)), Tick::Unchanged));
    }

    #[test]
    fn a_poll_that_answers_the_other_mode_is_one() {
        assert!(matches!(
            tick(Mode::Dark, Ok(Mode::Light)),
            Tick::Changed(Mode::Light)
        ));
    }

    #[test]
    fn one_change_fires_once_however_many_ticks_follow() {
        let mut current = Mode::Dark;
        let mut fired = Vec::new();

        // The registry flips once, and every later poll reads the new value back.
        for reading in [Mode::Dark, Mode::Light, Mode::Light, Mode::Light] {
            if let Tick::Changed(next) = tick(current, Ok(reading)) {
                current = next;
                fired.push(next);
            }
        }

        assert_eq!(fired, [Mode::Light]);
    }

    #[test]
    fn a_poll_that_cannot_read_is_neither_mode() {
        let reading = Err(wsl::Error::NoDrive);
        assert!(matches!(tick(Mode::Dark, reading), Tick::Unreadable(_)));
    }

    #[test]
    fn a_failure_says_which_one_it_was() {
        let unstartable = Failure::from(std::io::Error::from(std::io::ErrorKind::NotFound));
        assert!(unstartable.to_string().contains(SHELL), "{unstartable}");
    }
}
