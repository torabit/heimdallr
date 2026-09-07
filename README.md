<h1 align="center">heimdallr</h1>

<p align="center">
  <a href="https://github.com/torabit/heimdallr/actions"
    ><img
      src="https://img.shields.io/github/actions/workflow/status/torabit/heimdallr/ci.yml?branch=main&label=ci&style=flat-square"
      alt="CI status"
  /></a>
  <a href="https://github.com/torabit/heimdallr/blob/main/LICENSE-MIT"
    ><img
      src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square"
      alt="MIT OR Apache-2.0"
  /></a>
</p>

Your terminal tools do not follow the system theme. The OS flips to dark at sunset and every
config file you own stays where it was, until you go and change them.

**heimdallr watches the system light/dark theme and runs a command when it changes.** What the
command does is up to you.

```console
$ heimdallr
dark

$ heimdallr watch --on-change 'vanadis apply --variant "$1"'
```

- **Four platforms.** macOS, Windows and Linux come from
  [`dark-light`](https://github.com/rust-dark-light/dark-light). WSL reads the Windows theme
  through `reg.exe`, which nothing else does.
- **It will not guess.** A host with no system theme exits non-zero and says so, instead of
  reporting `light` and sending you off to apply a theme nobody asked for.
- **One word on stdout.** `$(heimdallr)` is a bare `dark` or `light`. Everything else goes to
  stderr.
- **Nothing to keep in step.** No config file, no daemon, no state of its own. `watch` fires
  once at start, so a machine that was asleep across the transition catches up.

## Status

Early. The command line is not implemented yet — see the
[issues](https://github.com/torabit/heimdallr/issues) for what is being built and in what
order. Nothing is published to crates.io.

Build from a clone:

```console
$ cargo build --release
```

## License

MIT or Apache-2.0, at your option.
