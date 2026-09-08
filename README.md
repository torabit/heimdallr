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

## Running it under WSL

`watch` runs from a `systemd --user` unit; a login shell is not needed. It does not need `PATH`
adjusting either. A systemd unit does not inherit the Windows directories WSL appends to a login
shell's `PATH`, so heimdallr finds `reg.exe` through `/proc/mounts` when it is not there.

WSL is the one platform that polls, because `reg.exe` has no notification to block on.
`--interval` sets how often, and defaults to 30 seconds:

```console
$ heimdallr watch --interval 60 --on-change 'vanadis apply --variant "$1"'
```

One read costs about 29 ms, so the default is around 0.1% of one core. The price is latency: a
switch lands up to one interval late. `--interval` is ignored everywhere else, where the system
says when the theme changed.

## Status

`heimdallr` prints the mode and `heimdallr watch` runs a command on every change, on all four
platforms. See the [issues](https://github.com/torabit/heimdallr/issues) for what is left.
Nothing is published to crates.io.

Build from a clone:

```console
$ cargo build --release
```

## License

MIT or Apache-2.0, at your option.
