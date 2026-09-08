# Recording the README's demo GIF

One recording, driven by [vhs](https://github.com/charmbracelet/vhs). Everything it reads and
writes lives under `demo/`, so a recording touches nothing in `~/.config` and nothing in the
herdr you are already running.

| tape | output | what it shows |
| --- | --- | --- |
| `demo.tape` | `media/demo.gif` | the Windows theme flipped from the command line, and an editor, a process viewer, a prompt and the terminal all following |

## It has to be recorded on WSL

vhs draws the terminal in a headless Chromium on the machine it runs on, so the recording sees
whatever system theme that machine has. On a Linux host with no XDG Desktop Portal `heimdallr`
only ever errors, and WSL cannot be faked there:
`kernel.apparmor_restrict_unprivileged_userns = 1` blocks the user namespace that a bind mount
over `/proc/sys/kernel/osrelease` would need.

WSL is also the platform worth showing. It is the one nothing else covers.

## Recording

From the repository root, and only from there. The tape uses relative paths, and so do the
reload commands in `demo/vanadis/config.toml`.

```sh
cargo build --release
PATH="$PWD/target/release:$PATH" vhs demo/demo.tape
```

`PATH` is for vhs's own `Require heimdallr`, which looks at the recorder's shell. The panes get
their copy from `demo/bin/demo-env`, which puts `target/release` ahead of `~/.cargo/bin` so
that a heimdallr installed on the recording machine is not what ends up in the GIF.

**The recording flips the real Windows theme.** The tape reads `AppsUseLightTheme` before it
writes anything, restores it in the teardown, and reads it back to check:

```
Wait+Screen@15s /restored=0/
```

A restore that did not happen fails the recording rather than being discovered on the desktop
afterwards. So does a value that could not be read in the first place, and that check comes
before the first write, when there is still nothing to undo.

### `VHS_NO_SANDBOX` is conditional, and this machine does not need it

Every write-up of vhs on Ubuntu says to set it. It is needed from Ubuntu 24.04, where AppArmor
blocks the unprivileged user namespace Chromium's zygote sandbox wants, and the failure is not
obvious: a stack trace ending in `content::ZygoteHostImpl::Init()` and the single line
`recording failed`.

```sh
sysctl kernel.apparmor_restrict_unprivileged_userns   # 1 on Ubuntu 24.04 and later
```

On the 22.04 this was recorded on the sysctl does not exist and the recording works without the
variable, which was measured rather than assumed: the same tape was recorded with and without
it. Set it if the sysctl reads `1`. Turning the sysctl off instead fixes it machine-wide and is
the worse trade.

## What has to be installed

`vhs`, and the four programs the GIF puts on screen. All five come from Homebrew.

```sh
brew install vhs herdr neovim btop starship
cargo install vanadis
```

vhs's own dependencies (ffmpeg, ttyd) come with it. What does not is the set of shared
libraries the Chromium it downloads at first run links against. Without them vhs prints
`error while loading shared libraries: libnss3.so` and nothing else:

```sh
sudo apt install libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 \
  libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libpango-1.0-0 libcairo2 \
  libasound2 libatspi2.0-0
```

### The font has to be the one the tape names

The tape names `JetBrains Mono`. Naming a font that is not installed is worth avoiding:
fontconfig answers with whatever it does have rather than an error, so the wrong name is
silent.

```sh
sudo apt install fonts-jetbrains-mono
fc-match "JetBrains Mono"   # must answer JetBrains Mono, not something else
```

On this machine, before that package, `fc-match "JetBrains Mono"` answered `DejaVu Sans Mono`,
which is at least monospaced. A name that lands on a full-width font is the worse case: it
doubles every cell, halves the grid, and reports nothing.

## How it is wired

```
demo/
  vanadis/          VANADIS_CONFIG. config.toml, four templates, two themes
  home/             XDG_CONFIG_HOME. herdr, nvim, btop and the shell read from here
  bin/              demo-env, btop-loop, reload-herdr, reload-nvim, follow-terminal-bg
```

Copied from [vanadis](https://github.com/torabit/vanadis)'s own `demo/` and changed in four
places: the session is `heimdallr-demo`, the nvim reload goes through a wrapper, the prompt
drops its git segments, and `demo-env` extends `PATH`. Each is explained below or in the file
it changed.

heimdallr is not what changes the colours. `vanadis apply --variant "$1"` is, and `[auto]` in
`demo/vanadis/config.toml` is what turns heimdallr's `dark` or `light` into a theme name. The
four targets follow that theme differently, and that difference is why there are four.

| target | how it follows a theme |
| --- | --- |
| starship | re-reads its config on every prompt, so nothing runs at all |
| herdr | `herdr server reload-config`, and the sidebar, tab bar, borders and pane backgrounds redraw |
| nvim | `:colorscheme vanadis` over its own RPC socket, which re-executes the generated file |
| btop | one `q`, because btop reads a theme once and `demo/bin/btop-loop` starts it again |

**The terminal follows too, over OSC.** Every cell no program has painted is drawn in the
terminal's default colours. In a real setup the terminal emulator is a vanadis target and reads
a config file like anything else. vhs reads none, and its `Set Theme` is applied once at
startup and ignored for the rest of the tape. What it does honour is OSC: vhs draws through
ttyd and xterm.js, which has handled OSC 10, 11 and 12 from the output stream since v5.0.
`demo/bin/follow-terminal-bg` is that reload command, and it has to run in the shell that owns
the terminal, started before `herdr session attach`, so that its stdout is the terminal.

## Four things that will bite

**`heimdallr watch &` is a background process group, and one of the reloads could not take
that.** `nvim --server … --remote-send` touches the terminal it was handed, which from a
background process group raises SIGTTOU and stops the whole job:

```
[1]+  Stopped     heimdallr watch --interval 1 --on-change 'vanadis apply --variant "$1"'
```

Nothing says why. heimdallr prints nothing on success, so a stopped watcher and a working one
look the same, and bash reports a stopped job at the next prompt rather than when it happened,
so the notice lands under a later command. The first recording of this tape had heimdallr
stopped by its own start-up run and a theme that never followed anything.

`demo/bin/reload-nvim` is the fix and its whole content is the redirection. vanadis's own demo
calls the same command straight from `config.toml` and is right to: there it is a child of a
typed `vanadis cycle`, running in the foreground.

Measured, with job control on and a live demo session: the bare command leaves its job in
state `T` and the wrapped one leaves it `Done`. `set -m` matters when reproducing this. A
non-interactive shell has job control off, `&` does not make a new process group, and nothing
gets SIGTTOU at all.

**`herdr server stop` is not how to end the attach.** Typed into a pane it does stop the
server, but the client did not exit inside three seconds, and every command after it went into
a pane of a session that was going away, including the one that restores the registry.
`prefix+q` — `Ctrl+b` then `q` — is herdr's detach, and it hands the recording shell its own
keyboard back. vanadis's tape does not notice the difference because it ends without waiting
for anything afterwards.

**`$ORIG` stays in the recording shell and the key goes to the pane.** After `herdr session
attach` the keystrokes reach the focused pane, so the shell pane needs its own copy of the
registry key. The original value deliberately does not follow it: the restore runs in the
recording shell after the detach, which is the only place the value read before any of this
was written still is.

**The prompt must not depend on the branch.** `demo/vanadis/templates/starship/starship.toml.in`
drops `$git_branch` and `$git_status`. This gets recorded on `torabit/docs/demo-gif`, which is
43 columns of prompt and wrapped every recorded command onto a second line. A committed GIF
whose layout depends on the branch it happened to be recorded from cannot be re-recorded to
look the same.

## Size and geometry

`Set Width 1800`, and the reason is the recorded commands rather than the panes. The cell is
9.2px at font size 14, measured off a recording, and the shell pane comes out 96 columns:
enough for the 93 of `torabit/heimdallr ❯ ` and the longest recorded command. vanadis's 1700
gives it about 90 and wraps that line.

btop is the other pane in the same column and refuses to draw below 80, printing "Terminal size
too small" instead. That is the floor. Shrinking the recording without looking at that pane is
how the TUI ends up blank.

The output is about 1.2MB over 23.6 seconds. Keep it under 2MB; lowering `Set Framerate` or
trimming a `Sleep` is the first thing to try, and the width and height are the last, for the
reason above.

## Checking a recording

The theme change is measurable, which beats squinting at frames:

```sh
ffprobe -v error -f lavfi -i "movie=media/demo.gif,signalstats" \
  -show_entries frame_tags=lavfi.signalstats.YAVG -of csv=p=0 \
  | awk '{n++; if (prev!="" && ((prev<130&&$1>=130)||(prev>=130&&$1<130))) \
      printf "transition at %.2fs: %.0f -> %.0f\n", (n-1)/24.0, prev, $1; prev=$1}'
```

The recording committed here answers two transitions, at 12.75s and 20.83s, which is dark to
light and back. A recording where the watcher was stopped answers none, and looks fine until
you ask.
