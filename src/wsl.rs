//! Reading the Windows theme from inside WSL.
//!
//! A WSL binary is a Linux binary, so `cfg(target_os)` cannot select this path and `dark_light`
//! compiles its freedesktop backend here. There is no XDG Desktop Portal under WSL reflecting the
//! Windows theme, so the branch has to happen at runtime, ahead of that backend.

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use thiserror::Error;

use crate::Mode;

/// Where the kernel release string is read from.
const OSRELEASE: &str = "/proc/sys/kernel/osrelease";

/// Where the Windows drives are found, whatever they are mounted on.
const MOUNTS: &str = "/proc/mounts";

/// The drive Windows is installed on, as `/proc/mounts` names it once unescaped. Matched without
/// regard to case, because nothing promises which one the kernel prints.
const WINDOWS_DRIVE: &str = r"C:\";

/// Under the drive's mount point, the directory `reg.exe` lives in.
const SYSTEM32: &str = "Windows/System32";

/// The registry value `dark_light`'s own Windows backend reads.
const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const VALUE: &str = "AppsUseLightTheme";

/// Why the Windows theme could not be read through WSL interop.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not run `{program}`: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`reg.exe` is not on PATH, and {MOUNTS} could not be read to find it: {0}")]
    Mounts(#[source] std::io::Error),
    #[error("`reg.exe` is not on PATH, and {MOUNTS} lists no Windows drive to find it under")]
    NoDrive,
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
    let output = query()?;

    if !output.status.success() {
        return Err(Error::Query {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    parse(&String::from_utf8_lossy(&output.stdout))
}

/// Runs `reg.exe query`, from PATH if it is there and from the Windows drive if it is not.
///
/// A `systemd --user` unit does not inherit the Windows directories WSL appends to a login
/// shell's PATH, and that is the run model `watch` is meant for. The fallback goes through
/// `/proc/mounts` rather than a hardcoded `/mnt/c` because `[automount] root` in `/etc/wsl.conf`
/// moves it, and the kernel already publishes where it went.
fn query() -> Result<Output, Error> {
    match run(OsStr::new("reg.exe")) {
        Err(absent) if absent.kind() == ErrorKind::NotFound => {
            let path = reg_exe()?;
            run(path.as_os_str()).map_err(|source| Error::Spawn {
                program: path.display().to_string(),
                source,
            })
        }
        found => found.map_err(|source| Error::Spawn {
            program: "reg.exe".to_owned(),
            source,
        }),
    }
}

fn run(program: &OsStr) -> std::io::Result<Output> {
    Command::new(program)
        .args(["query", KEY, "/v", VALUE])
        .output()
}

/// Where `reg.exe` is, for a process whose PATH does not have it.
fn reg_exe() -> Result<PathBuf, Error> {
    let mounts = std::fs::read_to_string(MOUNTS).map_err(Error::Mounts)?;
    drive_mount(&mounts)
        .map(|point| reg_exe_under(&point))
        .ok_or(Error::NoDrive)
}

/// `/mnt/c/Windows/System32/reg.exe`, for a drive mounted at `/mnt/c`.
fn reg_exe_under(point: &str) -> PathBuf {
    Path::new(point).join(SYSTEM32).join("reg.exe")
}

/// Where `/proc/mounts` says the Windows drive is mounted.
///
/// The kernel escapes space, tab, newline and backslash as octal in both fields, so `C:\` is
/// written `C:\134` and a mount point with a space in it carries `\040`.
fn drive_mount(mounts: &str) -> Option<String> {
    mounts.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let source = unescape(fields.next()?);
        let point = unescape(fields.next()?);
        source.eq_ignore_ascii_case(WINDOWS_DRIVE).then_some(point)
    })
}

/// Undoes `/proc/mounts`'s octal escaping. A backslash that does not start one stays as it is,
/// so a kernel that did not escape at all reads the same.
fn unescape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut rest = field.chars();

    while let Some(character) = rest.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        let octal: String = rest.clone().take(3).collect();
        match u8::from_str_radix(&octal, 8) {
            Ok(byte) if octal.len() == 3 => {
                out.push(char::from(byte));
                let _ = rest.nth(2);
            }
            _ => out.push('\\'),
        }
    }

    out
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

    /// `reg.exe query` output, byte for byte as observed on WSL2 through `| cat -A`:
    ///
    /// ```text
    /// ^M$
    /// HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize^M$
    ///     AppsUseLightTheme    REG_DWORD    0x0^M$
    /// ^M$
    /// ```
    fn capture(data: &str) -> String {
        format!(
            "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\r\n    {VALUE}    REG_DWORD    {data}\r\n\r\n"
        )
    }

    /// `/proc/mounts` on WSL2, with the backslash escaped the way the kernel escapes it.
    const WSL2_MOUNTS: &str = concat!(
        "/dev/sdd / ext4 rw,relatime,discard,errors=remount-ro,data=ordered 0 0\n",
        "none /dev devtmpfs rw,nosuid,relatime,mode=755 0 0\n",
        "C:\\134 /mnt/c 9p rw,dirsync,noatime,aname=drvfs;path=C:\\134;uid=1000 0 0\n",
        "D:\\134 /mnt/d 9p rw,dirsync,noatime 0 0\n",
    );

    #[test]
    fn reg_exe_sits_under_the_drive_mount() {
        assert_eq!(
            reg_exe_under("/mnt/c"),
            Path::new("/mnt/c/Windows/System32/reg.exe")
        );
    }

    #[test]
    fn the_windows_drive_is_found_where_it_is_mounted() {
        assert_eq!(drive_mount(WSL2_MOUNTS).as_deref(), Some("/mnt/c"));
    }

    #[test]
    fn a_moved_automount_root_is_followed() {
        let mounts = "C:\\134 /windows/c 9p rw 0 0\n";
        assert_eq!(drive_mount(mounts).as_deref(), Some("/windows/c"));
    }

    #[test]
    fn a_source_the_kernel_did_not_escape_reads_the_same() {
        let mounts = "C:\\ /mnt/c 9p rw 0 0\n";
        assert_eq!(drive_mount(mounts).as_deref(), Some("/mnt/c"));
    }

    #[test]
    fn an_escaped_space_in_the_mount_point_is_undone() {
        let mounts = "C:\\134 /mnt/my\\040drive 9p rw 0 0\n";
        assert_eq!(drive_mount(mounts).as_deref(), Some("/mnt/my drive"));
    }

    #[test]
    fn a_host_with_no_windows_drive_has_none() {
        let mounts = concat!(
            "sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0\n",
            "proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0\n",
            "/dev/nvme0n1p2 / ext4 rw,relatime,errors=remount-ro 0 0\n",
        );
        assert!(drive_mount(mounts).is_none());
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
