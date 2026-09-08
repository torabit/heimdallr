# The README's demo GIF. Run from the repository root:
#
#   cargo build --release
#   PATH="$PWD/target/release:$PATH" vhs demo/demo.tape
#
# It has to be recorded on WSL. `heimdallr watch` polls the Windows registry there, and the
# tape flips the registry itself, so the recording drives the real system theme rather than
# staging one. See demo/README.md.
#
# What is recorded is three commands: `heimdallr watch`, and two `reg.exe add` that are the
# command-line spelling of clicking the Windows appearance setting. heimdallr prints nothing
# on success, so what shows it fired is the output of the command it was given.
#
# Everything before `Show` builds the herdr session and is not recorded. The whole layout is
# built against a headless server and attached to at the very end. Typing the setup into the
# attached TUI instead loses it: herdr takes a while to become interactive, and keys sent
# before then go nowhere.

Output media/demo.gif

Require heimdallr
Require vanadis
Require herdr
Require nvim
Require btop
Require starship

Set Shell "bash"

# Not `JetBrainsMono Nerd Font` unless one is installed. fontconfig answers a name it does
# not have with whatever it does have, and no error, so a wrong name is silent: on this
# machine `fc-match "JetBrains Mono"` answered `DejaVu Sans Mono` until the font was
# installed, and a name that lands on a full-width font doubles every cell.
Set FontFamily "JetBrains Mono"
Set FontSize 14
Set Width 1800
Set Height 950
Set Padding 0
Set Framerate 24
Set TypingSpeed 45ms

# 1800 and not vanadis's 1700, for the recorded commands rather than for btop. Measured off
# a recording: the cell is 9.2px at font size 14 and the shell pane comes out 96 columns
# wide, which takes the 93 of `torabit/heimdallr ❯ ` and the longest recorded command with
# three to spare. 1700 gives it about 90 and wraps that line.
#
# btop is the other pane in the same column and refuses to draw below 80, printing
# "Terminal size too small" instead, so this is the floor and not the ceiling: shrinking the
# recording without looking at that pane is how the TUI ends up blank.

# The seed, so nothing flashes before the attach, and the ANSI palette for the whole
# recording: `Set Theme` is applied once and ignored afterwards, which is why
# demo/bin/follow-terminal-bg exists. The terminal does follow, over OSC, and not from here.
# This is papercolor-dark, which is where the recording starts.
Set Theme { "name": "papercolor-dark", "background": "#1c1c1c", "foreground": "#d0d0d0", "cursor": "#5fafd7", "selection": "#3a3a3a", "black": "#1c1c1c", "red": "#af005f", "green": "#5faf00", "yellow": "#d7af5f", "blue": "#5fafd7", "magenta": "#af87d7", "cyan": "#ffaf00", "white": "#d0d0d0", "brightBlack": "#808080", "brightRed": "#d7005f", "brightGreen": "#ffd75f", "brightYellow": "#87d75f", "brightBlue": "#ffd700", "brightMagenta": "#87d7ff", "brightCyan": "#87d700", "brightWhite": "#c6c6c6" }

Hide

# The demo's environment, for the commands typed here. Every pane goes through
# demo/bin/demo-env instead, because the pane shells read the recorder's rc files and put
# their own values back.
Type "export COLORTERM=truecolor" Enter
Type "export VANADIS_CONFIG=$PWD/demo/vanadis" Enter
Type "export XDG_CONFIG_HOME=$PWD/demo/home" Enter
Type "export XDG_DATA_HOME=$PWD/demo/home/share" Enter
Type "export XDG_STATE_HOME=$PWD/demo/home/state" Enter
Type "export STARSHIP_CONFIG=$PWD/demo/home/starship.toml" Enter
Type "export HERDR_CONFIG_PATH=$PWD/demo/home/herdr/config.toml" Enter
Type "export PATH=/mnt/c/Windows/System32:$PATH" Enter
Wait

# The recorder's own theme, read before anything is written, and restored in the teardown.
# This flips the real desktop: the value here is what it goes back to.
Type `KEY='HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'` Enter
Wait
Type `ORIG=$(reg.exe query "$KEY" /v AppsUseLightTheme | tr -d '\r' | awk '/AppsUseLightTheme/ {print $3}'); echo "ORIG=$ORIG"` Enter

# Not a courtesy. A tape that cannot read the value has nothing to restore, and this is the
# last moment at which that is still true: nothing has been written yet. `Wait` that times
# out aborts the tape instead of leaving the desktop wherever the recording put it.
Wait+Screen@15s /ORIG=0x[01]/

# The recording starts dark whatever the recorder's desktop was on, so the GIF loops without
# a seam and the first frame is not a matter of when it was recorded.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 0 /f >/dev/null 2>&1` Enter
Wait

# The starting state on disk. Nothing is running yet, so every reload fails and apply exits
# non-zero; the files it would have reloaded are written, which is all this needs.
Type "vanadis apply --variant dark >/dev/null 2>&1; true" Enter
Wait

# A session that already exists keeps its panes, and every new run splits three more onto
# the end. The btop target names a pane by id, so the run has to start from nothing. A
# socket left behind by a killed recording stops the next nvim from listening on it.
Type "herdr --session heimdallr-demo server stop >/dev/null 2>&1; herdr session delete heimdallr-demo >/dev/null 2>&1; rm -f $PWD/demo/home/nvim.sock; true" Enter
Wait

Type "(setsid herdr --session heimdallr-demo server >/dev/null 2>&1 </dev/null &)" Enter
Wait

# The server takes a moment to open its socket, and `workspace create` is the first command
# that needs it. Retrying until it answers beats guessing how long that is.
Type "until herdr --session heimdallr-demo pane list >/dev/null 2>&1; do sleep 0.2; done" Enter
Wait@30s

# A headless server starts with no workspace at all, so the root pane has to be asked for.
Type "herdr --session heimdallr-demo workspace create --cwd $PWD --label heimdallr >/dev/null" Enter
Wait

# Left pane the editor, right column the TUI over the shell.
Type "herdr --session heimdallr-demo pane split w1:p1 --direction right --ratio 0.42 --cwd $PWD --no-focus >/dev/null" Enter
Wait
Type "herdr --session heimdallr-demo pane split w1:p2 --direction down --ratio 0.55 --cwd $PWD --no-focus >/dev/null" Enter
Wait

# These four stay on Sleep. `Wait` watches the shell this is typed into, and what has to
# happen is a program drawing inside a pane, which that shell cannot report.
Type "herdr --session heimdallr-demo pane run w1:p2 demo/bin/demo-env demo/bin/btop-loop" Enter
Sleep 2s
Type "herdr --session heimdallr-demo pane run w1:p3 demo/bin/demo-env bash --rcfile demo/home/bashrc" Enter
Sleep 2s
Type "herdr --session heimdallr-demo pane run w1:p3 clear" Enter
Sleep 1s

# src/watch.rs and not a README: it is the poll the tape is about to trigger, and Rust gives
# the colorscheme keywords, strings and comments to colour.
Type "herdr --session heimdallr-demo pane run w1:p1 demo/bin/demo-env nvim --listen $PWD/demo/home/nvim.sock src/watch.rs" Enter
Sleep 3s
Type "herdr --session heimdallr-demo pane focus --pane w1:p2 --direction down >/dev/null" Enter
Wait

# The terminal's own default colours, which no pane can reach. This has to start here, in
# the shell that owns the terminal, so that its stdout is the terminal. See the script.
Type "demo/bin/follow-terminal-bg &" Enter
Wait

# Not a Sleep. herdr is not interactive for several seconds after the attach. `grouped` is a
# label herdr only draws once its sidebar is up.
Type "clear; herdr session attach heimdallr-demo" Enter
Wait+Screen@30s /grouped/
Sleep 2s

# From here the keystrokes reach the focused pane and not this shell, so the shell pane needs
# its own copy of the key. `$ORIG` deliberately stays behind: the restore runs in this shell
# after the attach exits, where the value read before any of this was written still is.
Type `KEY='HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'` Enter
Sleep 1s
Type "clear" Enter
Sleep 1s

Show

# Dark. Let it be read before anything moves.
Sleep 1500ms

# heimdallr prints nothing when a run succeeds, so the three lines that appear are vanadis's,
# through heimdallr's stdout. `--interval 1` because the default 30 would be 30 seconds of
# GIF spent waiting; the flag is real and this is what it is for.
#
# It fires once here, with the mode as it already is, which is the "it catches up" the README
# claims: a machine that was off across the transition is not left behind. Nothing changes
# colour, because nothing is out of date.
Type `heimdallr watch --interval 1 --on-change 'vanadis apply --variant "$1"' &` Enter
Sleep 3s

# Light. This is the Windows appearance setting, typed rather than clicked, and it changes
# the real desktop.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 1 /f >/dev/null` Enter
Sleep 4s

# And back, so the loop has no seam.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 0 /f >/dev/null` Enter
Sleep 4s

Hide

# Typed into the pane while the attach is still in the foreground.
Type "kill %1" Enter
Sleep 1s

# `prefix+q`, which is herdr's detach, and the only thing here that reliably hands this
# shell its own keyboard back. Typing `herdr server stop` into the pane instead does stop
# the server, but the client did not exit inside three seconds and every command after it
# went into a pane of a session that was going away; the `Wait` that follows is what caught
# it, and vanadis's tape does not notice only because it ends without waiting.
Ctrl+b
Sleep 500ms
Type "q"
Sleep 2s
Wait

Type "herdr --session heimdallr-demo server stop" Enter
Wait
Type "herdr session delete heimdallr-demo" Enter
Wait

# The recorder's desktop, back where it was. Reading `$ORIG` here is the whole reason it was
# never sent to the pane.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d $((ORIG)) /f >/dev/null 2>&1` Enter
Wait
Type `kill %1 2>/dev/null; true` Enter
Wait

# Said out loud, so a restore that did not happen fails the recording rather than being
# discovered on the desktop afterwards.
#
# `$?` and not the word itself: `Wait+Screen` matches the whole screen, and a command that
# contains the string it is waiting for is matched by the typing rather than by the result.
Type `reg.exe query "$KEY" /v AppsUseLightTheme | tr -d '\r' | grep -qw "$ORIG"; echo "restored=$?"` Enter
Wait+Screen@15s /restored=0/
