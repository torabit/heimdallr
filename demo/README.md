# Recording the README's demo GIF

One recording, driven by [vhs](https://github.com/charmbracelet/vhs). Everything it reads and
writes lives under `demo/`, so a recording touches nothing in `~/.config`.

| tape | output | what it shows |
| --- | --- | --- |
| `demo.tape` | `media/demo.gif` | the Windows theme set from the command line, and the prompt and the terminal following within a second |

One terminal and no multiplexer, which was the second attempt. The first ran a four-pane herdr
session with Neovim and btop following the theme as well, and at the width four panes need,
none of it was legible once the GIF was scaled into a README column. Everything that is on
screen now can be read.

## It has to be recorded on WSL

vhs draws the terminal in a headless Chromium on the machine it runs on, so the recording sees
whatever system theme that machine has. On a Linux host with no XDG Desktop Portal `heimdallr`
only ever errors, and WSL cannot be faked there:
`kernel.apparmor_restrict_unprivileged_userns = 1` blocks the user namespace that a bind mount
over `/proc/sys/kernel/osrelease` would need.

WSL is also the platform worth showing. It is the one nothing else covers.

## Recording

From the repository root, and only from there. The tape uses relative paths.

```sh
cargo build --release
PATH="$PWD/target/release:$PATH" vhs demo/demo.tape
```

`PATH` is for vhs's own `Require heimdallr`, which looks at the recorder's shell. The tape
exports its own copy, with `target/release` ahead of `~/.cargo/bin` so that a heimdallr
installed on the recording machine is not what ends up in the GIF.

### The recording flips the real Windows theme

The tape reads `AppsUseLightTheme` before it writes anything and restores it in the teardown.
The read is checked first:

```
Wait+Screen@15s /ORIG=0x[01]/
```

A value that could not be read aborts the tape at the last moment when there is still nothing
to undo. The restore is checked by its own exit status, `restore=0`.

**It is not read back inside the tape, and that is deliberate.** An earlier version ran a
second `reg.exe query` to compare, and that query intermittently never returned: `restore=0`
printed and then nothing, until the `Wait` timed out on a command that had not come back. Two
recordings out of three, with `grep -q` and with an `awk` that reads its input to the end
alike, so the first guess — `-q` closing the pipe under a writer — was wrong and the mechanism
was never established. `reg.exe add` has returned every time. So check it afterwards instead:

```sh
reg.exe query 'HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize' \
  /v AppsUseLightTheme
```

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

```sh
brew install vhs starship
cargo install vanadis
sudo apt install fonts-jetbrains-mono
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
fontconfig answers with whatever it does have rather than an error, so a wrong name is silent.

```sh
fc-match "JetBrains Mono"   # must answer JetBrains Mono, not something else
```

On this machine, before `fonts-jetbrains-mono`, it answered `DejaVu Sans Mono`, which is at
least monospaced. A name that lands on a full-width font is the worse case: it doubles every
cell, halves the grid, and reports nothing.

## How it is wired

```
demo/
  vanadis/          VANADIS_CONFIG. config.toml, one template, two themes
  home/             XDG_CONFIG_HOME. bashrc, and the starship.toml vanadis renders
  bin/              follow-terminal-bg
```

Copied from [vanadis](https://github.com/torabit/vanadis)'s own `demo/` and cut to what one
terminal can show. The themes were cut too, and not to the nine tokens this demo reads:

```sh
vanadis check   # must say `checked 1 target` and name no undefined core tokens
```

`check` calls 33 tokens core — all 17 of `[role]` and all 16 of `[ansi]` — and reports a
theme that leaves any of them out against every target it renders. Trimming to what is read
made it report 24. What went instead is what is dead and not core: the `[diff]` table, which
existed for a diff pager, the `[text]` table, whose values are other tools' own theme names,
and the primitives that only the nvim and herdr templates referenced by appearance.

heimdallr is not what changes the colours. `vanadis apply --variant "$1"` is, and `[auto]` in
`demo/vanadis/config.toml` is what turns heimdallr's `dark` or `light` into a theme name, so
the README's headline command needs nothing between the two tools.

| what follows the theme | how |
| --- | --- |
| the prompt | starship re-reads its config on every prompt, so nothing runs at all |
| the terminal's own colours | OSC 10, 11 and 12 from `demo/bin/follow-terminal-bg` |

vhs reads no config file and applies `Set Theme` once at startup, so the terminal cannot be a
vanadis target the way a real emulator is. What it does honour is OSC: it draws through ttyd
and xterm.js, which has handled OSC 10, 11 and 12 from the output stream since v5.0.
`demo/bin/follow-terminal-bg` is that reload command, and it has to be the process whose
stdout is the terminal.

**heimdallr prints nothing when a run succeeds**, so what shows it fired is the output of the
command it was given: `applied papercolor-light` and `wrote starship`.

## Five things that will bite

**`heimdallr watch &` is a background process group, and that limits what `--on-change` can
be.** A command whose children touch the terminal gets SIGTTOU and stops the whole job:

```
[1]+  Stopped     heimdallr watch --interval 1 --on-change 'vanadis apply --variant "$1"'
```

Nothing says why. heimdallr prints nothing on success, so a stopped watcher and a working one
look the same, and bash reports a stopped job at the next prompt rather than when it happened,
so the notice lands under a later command. The four-pane cut of this tape hit it: `nvim
--server … --remote-send` was one of the reloads, and the watcher was stopped by its own
start-up run, with a recording to show for it in which nothing ever changed colour.

Measured, with job control on: the bare command leaves its job in state `T` and the same
command with its output redirected leaves it `Done`. `set -m` matters when reproducing this. A
non-interactive shell has job control off, `&` does not make a new process group, and nothing
gets SIGTTOU at all.

Nothing in the current `--on-change` touches the terminal. This is also the reason the tape
waits on `applied` rather than sleeping through it:

```
Wait+Screen@20s /papercolor-dark/
```

A tape that gets no theme change aborts instead of producing a plausible GIF.

**Do not kill the watcher in the teardown.** `kill %1` signals the job's whole process group,
and that group holds whatever `reg.exe` heimdallr had running at the time. The shell stopped
answering and `echo watcher=$?` never printed. Nothing has to be killed: both background jobs
get SIGHUP when vhs takes the shell away. Check with `pgrep -af
"follow-terminal-bg|heimdallr watch"` after a recording, which is how the version that
disowned `follow-terminal-bg` was caught leaving it running.

**`Sleep` where a `Wait` would do is how a broken recording gets committed.** Every bare `Wait`
in the setup is the default `/>$/` against the last line, which is vhs's own `> ` prompt, and
`source demo/home/bashrc` is the line that ends them: from there the prompt is `heimdallr ❯`
and that pattern has nothing to match. Matching the new prompt does not work either — it is on
the line being typed as well as the line after, so the wait is satisfied before the command
has run. Every step after it waits on output the command itself produces, `echo prompt=$?` and
`echo restore=$?` and the rest, with `$?` and never the word, because `Wait+Screen` matches
the whole screen and a command containing the string it waits for is matched by its own typing.

The `Sleep`s that are left are all in the recorded section, where the point is a duration on
screen, plus one after `clear`, where what has to be true is that the screen is empty and no
regexp says that.

One transition cannot be waited on: `papercolor-dark` is on screen from the start-up run, so
the flip back has nothing to match that has not matched already. The teardown checks that one
through `vanadis current`, which knows which theme was applied last rather than which words
are on the screen.

**Padding and leftover height do not follow the theme.** vhs paints both from `Set Theme`,
once, and OSC 11 reaches only the cells inside xterm.js. `Set Padding 20` was 20px of
papercolor-dark framing a light terminal for a third of the recording, and `Set Height 560`
left a 12px bar above the grid and a 15px bar below it. `Padding 0`, and a height that a whole
number of rows fills:

```sh
ffmpeg -i frame.png -vf "crop=1:534:5:0,format=gray" -f rawvideo col.raw
```

and then the first and last row in `col.raw` that is not the seed background. That said 533,
and the tape uses 534, because an odd height fails in ffmpeg rather than in vhs and fails
after the recording: `Failed to configure input pad`, a zero-byte GIF, and vhs still exiting
0. Rounding down drops a whole row and puts 29px of bar back.

**The prompt must not carry the branch name.** `demo/vanadis/templates/starship/starship.toml.in`
drops `$git_branch` and `$git_status`. This gets recorded on `torabit/docs/demo-gif`, which is
43 columns of prompt and wrapped every recorded command onto a second line. A committed GIF
whose layout depends on the branch it happened to be recorded from cannot be re-recorded to
look the same.

## Size and geometry

Font size first and width second, which is the opposite of the four-pane cut. The cell is
0.657px per point of font size, measured off a recording, so 22pt is 14.5px and `Set Width
1400` leaves 96 columns. The longest line on screen is 85: `heimdallr ❯ ` and the `heimdallr
watch` command. Scaled to the 900px the README asks for, that is a 14px effective font, which
is the whole point of this shape.

`truncation_length` is 1 rather than vanadis's 2 for the same arithmetic. Every column the
prompt does not take is a column that goes into font size instead.

The output is about 280KB over 23.2 seconds. Keep it under 2MB; lowering `Set Framerate` or
trimming a `Sleep` is the first thing to try.

## Checking a recording

The theme change is measurable, which beats squinting at frames:

```sh
ffprobe -v error -f lavfi -i "movie=media/demo.gif,signalstats" \
  -show_entries frame_tags=lavfi.signalstats.YAVG -of csv=p=0 \
  | awk '{n++; if (prev!="" && ((prev<130&&$1>=130)||(prev>=130&&$1<130))) \
      printf "transition at %.2fs: %.0f -> %.0f\n", (n-1)/24.0, prev, $1; prev=$1}'
```

The recording committed here answers two transitions, at 11.54s and 17.88s, which is dark to
light and back. A recording where the watcher was stopped answers none, and looks fine until
you ask.
