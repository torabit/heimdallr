//! Reads the system light/dark mode, and refuses to guess when it cannot.
//!
//! The macOS, Windows and Linux backends are
//! [`dark_light`]'s. What this crate adds is WSL, where none of them applies, and the third
//! outcome: a mode that could not be determined is an error carrying why, never a default.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::fmt;

use thiserror::Error;

pub mod wsl;

/// The system theme, once it is known.
///
/// There is no `Unspecified`. A mode that could not be determined is an [`Undetectable`], so a
/// caller holding a `Mode` holds an answer the system actually gave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Dark => "dark",
            Self::Light => "light",
        })
    }
}

/// Why the system theme could not be determined.
///
/// Every variant is a reason a caller can act on. None of them is a mode.
#[derive(Debug, Error)]
pub enum Undetectable {
    /// The desktop answered, and its answer was that it has no preference. A headless host and a
    /// desktop set to neither are the same to us and neither is light.
    #[error("the system reports no light or dark preference")]
    Unspecified,
    /// Detection ran and failed. The backend says what it was doing when it did.
    #[error("could not read the system theme: {0}")]
    Backend(#[from] dark_light::Error),
    /// Interop with Windows failed, on a host where that is the only way to ask.
    #[error("could not read the Windows theme from WSL: {0}")]
    Wsl(#[from] wsl::Error),
}

/// Reads the system theme.
///
/// # Errors
///
/// Returns [`Undetectable`] when the platform has no theme to report, or when reading it failed.
pub fn detect() -> Result<Mode, Undetectable> {
    if wsl::active() {
        return Ok(wsl::detect()?);
    }
    Mode::try_from(dark_light::detect()?)
}

impl TryFrom<dark_light::Mode> for Mode {
    type Error = Undetectable;

    /// `Unspecified` is not rounded to light. It is the whole reason this conversion can fail.
    fn try_from(mode: dark_light::Mode) -> Result<Self, Undetectable> {
        match mode {
            dark_light::Mode::Dark => Ok(Self::Dark),
            dark_light::Mode::Light => Ok(Self::Light),
            dark_light::Mode::Unspecified => Err(Undetectable::Unspecified),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_survive_the_conversion() {
        assert_eq!(Mode::try_from(dark_light::Mode::Dark).unwrap(), Mode::Dark);
        assert_eq!(
            Mode::try_from(dark_light::Mode::Light).unwrap(),
            Mode::Light
        );
    }

    #[test]
    fn unspecified_is_not_light() {
        let outcome = Mode::try_from(dark_light::Mode::Unspecified);
        assert!(matches!(outcome, Err(Undetectable::Unspecified)));
    }

    #[test]
    fn a_mode_prints_as_one_bare_word() {
        assert_eq!(Mode::Dark.to_string(), "dark");
        assert_eq!(Mode::Light.to_string(), "light");
    }

    #[test]
    fn an_undetectable_says_what_failed() {
        let backend = Undetectable::from(dark_light::Error::XdgDesktopPortal("no session".into()));
        let message = backend.to_string();
        assert!(message.contains("no session"), "{message}");
    }
}
