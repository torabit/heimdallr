//! Reading the Windows theme from inside WSL.
//!
//! A WSL binary is a Linux binary, so `cfg(target_os)` cannot select this path and `dark_light`
//! compiles its freedesktop backend here. There is no XDG Desktop Portal under WSL reflecting the
//! Windows theme, so the branch has to happen at runtime, ahead of that backend.

use std::process::Command;

use thiserror::Error;

use crate::Mode;

/// Where the kernel release string is read from.
const OSRELEASE: &str = "/proc/sys/kernel/osrelease";

/// The registry value `dark_light`'s own Windows backend reads.
const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const VALUE: &str = "AppsUseLightTheme";

/// Why the Windows theme could not be read through WSL interop.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not run `reg.exe`: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("`reg.exe query {KEY} /v {VALUE}` exited with {status}: {stderr}")]
    Query {
        status: std::process::ExitStatus,
        stderr: String,
    },
    #[error("`reg.exe` printed no {VALUE} value:\n{output}")]
    Missing { output: String },
    #[error("{VALUE} is {value}, which is neither 0x0 (dark) nor 0x1 (light)")]
    Unexpected { value: String },
}

/// Whether a kernel release string is WSL's.
///
/// The file is read rather than `$WSL_DISTRO_NAME` because a process started by a systemd unit
/// may not carry the variable.
#[must_use]
pub fn is_wsl(osrelease: &str) -> bool {
    osrelease.to_ascii_lowercase().contains("microsoft")
}

/// Whether this process is running under WSL.
#[must_use]
pub fn active() -> bool {
    std::fs::read_to_string(OSRELEASE).is_ok_and(|release| is_wsl(&release))
}

/// Reads the Windows theme by shelling out to `reg.exe`.
///
/// # Errors
///
/// Returns [`Error`] when `reg.exe` cannot be run, refuses the query, or answers with something
/// that is not a mode. None of those is light.
pub fn detect() -> Result<Mode, Error> {
    let output = Command::new("reg.exe")
        .args(["query", KEY, "/v", VALUE])
        .output()
        .map_err(Error::Spawn)?;

    if !output.status.success() {
        return Err(Error::Query {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    parse(&String::from_utf8_lossy(&output.stdout))
}

/// Reads the mode out of `reg.exe query` output.
///
/// The output is CRLF, padded with blank lines, and separates the value name, its type and its
/// data with runs of spaces. Splitting on whitespace absorbs all three.
///
/// # Errors
///
/// Returns [`Error::Missing`] when no line names the value, and [`Error::Unexpected`] when its
/// data is not one of the two documented values.
pub fn parse(output: &str) -> Result<Mode, Error> {
    let data = output
        .lines()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            (fields.next() == Some(VALUE)).then(|| fields.next_back())?
        })
        .ok_or_else(|| Error::Missing {
            output: output.trim().to_owned(),
        })?;

    match data
        .strip_prefix("0x")
        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
    {
        Some(0) => Ok(Mode::Dark),
        Some(1) => Ok(Mode::Light),
        _ => Err(Error::Unexpected {
            value: data.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `reg.exe query` output: CRLF, a leading blank line, the key path, the value line indented
    /// and space-padded, and a trailing blank line.
    fn capture(data: &str) -> String {
        format!(
            "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\r\n    {VALUE}    REG_DWORD    {data}\r\n\r\n"
        )
    }

    #[test]
    fn the_wsl_kernel_release_is_recognised() {
        assert!(is_wsl("6.18.33.2-microsoft-standard-WSL2"));
    }

    #[test]
    fn a_native_linux_kernel_release_is_not() {
        assert!(!is_wsl("7.0.0-30-generic"));
    }

    #[test]
    fn zero_is_dark_and_one_is_light() {
        assert!(matches!(parse(&capture("0x0")), Ok(Mode::Dark)));
        assert!(matches!(parse(&capture("0x1")), Ok(Mode::Light)));
    }

    #[test]
    fn a_value_that_is_neither_is_not_rounded_to_light() {
        let outcome = parse(&capture("0x2"));
        assert!(
            matches!(outcome, Err(Error::Unexpected { .. })),
            "{outcome:?}"
        );
    }

    #[test]
    fn output_without_the_value_is_missing_not_light() {
        let outcome = parse("\r\nHKEY_CURRENT_USER\\Software\r\n\r\n");
        assert!(matches!(outcome, Err(Error::Missing { .. })), "{outcome:?}");
    }

    #[test]
    fn an_error_names_the_value_it_could_not_read() {
        let message = parse(&capture("0x2")).unwrap_err().to_string();
        assert!(message.contains("0x2"), "{message}");
        assert!(message.contains(VALUE), "{message}");
    }
}
