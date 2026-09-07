# heimdallr

## What it is

Watches the system light/dark theme and runs a command when it changes.

The macOS, Windows and Linux backends are
[`dark-light`](https://github.com/rust-dark-light/dark-light)'s, and stay that way. What
heimdallr adds is WSL, where no portal reflects the Windows theme, and a watcher that runs a
command. A fix that belongs in a backend belongs upstream.

Three decisions follow, and none of them is negotiable.

**It holds no state.** The command it runs owns "which theme is applied"; a second record of
that would drift. Being called twice with the same mode is harmless, so `watch` fires once at
start rather than waiting for a change it may already have missed.

**Staying alive is not its job.** No config file, no daemon, no PID file, no log rotation.
systemd and launchd do that, and the unit files belong in the README rather than in code.

**The output is for a shell.** Bare `heimdallr` prints one word on stdout and nothing else, so
`$(heimdallr)` is usable. Anything that is not the mode goes to stderr.

## Rules

Machine-checkable rules are gates, not documents. Run the CI gates locally before pushing.
Once a lint covers a rule, delete it from `.claude/rules/`.

`.claude/rules/` carries only what no tool can check, and only problems this codebase has. A
rule describing a problem that does not exist here is the same failure as a rule no tool
enforces.

No dependency ahead of the work that needs it.

## Working an issue

1. `Depends on #N` is binding.
2. `spike` issues report an observation and keep no code.
3. Branch as `torabit/<type>/<slug>`.
4. Open a pull request referencing the issue.

## Commits, issues and pull requests

Written in English.

Conventional Commits: `type(scope): description`.

- Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `style`, `perf`, `ci`.
- Subject imperative, lowercase, no trailing period, at most 72 characters.
- Body explains why, when that is not self-evident.
