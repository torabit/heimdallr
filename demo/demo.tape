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
# One terminal and no multiplexer. An earlier cut of this ran a four-pane herdr session with
# nvim and btop following the theme as well, and at the width that took, none of it was
# legible once the GIF was scaled into a README column. What follows the theme here is the
# prompt and the terminal's own colours, and both can be read.

Output media/demo.gif

Require heimdallr
Require vanadis
Require starship

Set Shell "bash"

# Not `JetBrainsMono Nerd Font` unless one is installed. fontconfig answers a name it does
# not have with whatever it does have, and no error, so a wrong name is silent: on this
# machine `fc-match "JetBrains Mono"` answered `DejaVu Sans Mono` until the font was
# installed, and a name that lands on a full-width font doubles every cell.
Set FontFamily "JetBrains Mono"
Set FontSize 22
Set Width 1400
Set Height 534
Set Padding 0
Set Framerate 24
Set TypingSpeed 45ms

# Font size first and width second, which is the opposite of the earlier four-pane cut. The
# cell is 0.657px per point of font size, measured off a recording, so 22pt is 14.5px and
# 1400 leaves 96 columns. The longest line on screen is 85: `heimdallr ❯ ` and the
# `heimdallr watch` command. Scaled to the 900px the README asks for, that is a 14px
# effective font, which is the whole reason this shape was chosen.
#
# The prompt is one directory component rather than vanadis's two for the same arithmetic.
# Every column the prompt does not take is a column that goes into font size instead.
#
# `Padding 0` because padding does not follow the theme. vhs paints it from `Set Theme`,
# once, and OSC 11 reaches only the cells inside xterm.js, so 20px of padding was 20px of
# papercolor-dark drawn as a frame around a light terminal for a third of the recording.
#
# The height is the same problem, and 534 is a measured number rather than a chosen one.
# Whatever a whole number of rows does not fill gets that same fixed colour, and at 560 that
# was a 12px bar above the terminal and a 15px bar below it, dark, across the light half of
# the recording. What the terminal actually occupied there was 533px:
#
#   ffmpeg -i frame.png -vf "crop=1:H:5:0,format=gray" -f rawvideo col.raw
#
# and then the first and last row in `col.raw` that is not the seed background. 534 and not
# 533 because an odd height fails in ffmpeg rather than in vhs, and it fails after the
# recording: `Failed to configure input pad`, a zero-byte GIF, and vhs still exiting 0.
# Rounding down instead would drop a whole row and put 29px of bar back.

# The seed, so nothing flashes before the first frame, and the ANSI palette for the whole
# recording: `Set Theme` is applied once and ignored afterwards, which is why
# demo/bin/follow-terminal-bg exists. The terminal does follow, over OSC, and not from here.
# This is papercolor-dark, which is where the recording starts.
Set Theme { "name": "papercolor-dark", "background": "#1c1c1c", "foreground": "#d0d0d0", "cursor": "#5fafd7", "selection": "#3a3a3a", "black": "#1c1c1c", "red": "#af005f", "green": "#5faf00", "yellow": "#d7af5f", "blue": "#5fafd7", "magenta": "#af87d7", "cyan": "#ffaf00", "white": "#d0d0d0", "brightBlack": "#808080", "brightRed": "#d7005f", "brightGreen": "#ffd75f", "brightYellow": "#87d75f", "brightBlue": "#ffd700", "brightMagenta": "#87d7ff", "brightCyan": "#87d700", "brightWhite": "#c6c6c6" }

Hide

# Nothing outside demo/ is read or written. `target/release` ahead of everything else is what
# keeps a heimdallr installed on the recording machine out of the GIF, and System32 is what
# makes the recorded command a bare `reg.exe`, which is what a WSL install with the default
# `[interop] appendWindowsPath = true` gives you. This machine has it off, and the absolute
# path is 96 characters that say nothing.
Type "export PATH=$PWD/target/release:/mnt/c/Windows/System32:$PATH" Enter
Type "export COLORTERM=truecolor" Enter
Type "export VANADIS_CONFIG=$PWD/demo/vanadis" Enter
Type "export XDG_CONFIG_HOME=$PWD/demo/home" Enter
Type "export XDG_DATA_HOME=$PWD/demo/home/share" Enter
Type "export XDG_STATE_HOME=$PWD/demo/home/state" Enter
Type "export STARSHIP_CONFIG=$PWD/demo/home/starship.toml" Enter
Wait

# The recorder's own theme, read before anything is written, and restored in the teardown.
# This flips the real desktop: the value here is what it goes back to.
Type `KEY='HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'` Enter
Wait
Type `ORIG=$(reg.exe query "$KEY" /v AppsUseLightTheme | tr -d '\r' | awk '/AppsUseLightTheme/ {print $3}'); echo "ORIG=$ORIG"` Enter

# Not a courtesy. A tape that cannot read the value has nothing to restore, and this is the
# last moment at which that is still true: nothing has been written yet. A `Wait` that times
# out aborts the tape instead of leaving the desktop wherever the recording put it.
Wait+Screen@15s /ORIG=0x[01]/

# The recording starts dark whatever the recorder's desktop was on, so the GIF loops without
# a seam and the first frame is not a matter of when it was recorded.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 0 /f >/dev/null 2>&1` Enter
Wait

# The starting state on disk, so the prompt is themed before the first frame.
Type "vanadis apply --variant dark >/dev/null" Enter
Wait

# The terminal's own default colours. This has to be the process whose stdout is the
# terminal, which is why it starts here and not from a reload command.
#
# Left in the job table rather than disowned, which is why the recorded `heimdallr watch &`
# announces itself as `[2]` and not `[1]`. Disowning it to get the nicer number leaves it
# running after the recording: nothing in the teardown can be trusted to kill it, and
# nothing has to, because a job of this shell gets SIGHUP when vhs takes the shell away.
Type "demo/bin/follow-terminal-bg &" Enter
Wait

# Sourced and not exec'd: `$KEY` and `$ORIG` have to survive into the teardown.
#
# Every bare `Wait` above is the default `/>$/` against the last line, which is vhs's own
# `> ` prompt, and this is the line that ends them. From here the prompt is `heimdallr ❯`
# and there is nothing for that pattern to match. Matching the new prompt instead does not
# work either: it is on the line being typed as well as the line after, so the wait is
# satisfied before the command has run. Every step from here waits on output the command
# itself produces.
Type "source demo/home/bashrc; echo prompt=$?" Enter
Wait+Screen@15s /prompt=0/

# The one place a `Sleep` is the honest instrument. What has to be true is that the screen
# is empty, which no regexp says, and `clear` is instant.
Type "clear" Enter
Sleep 500ms

Show

# Dark. Let it be read before anything moves.
Sleep 1500ms

# heimdallr prints nothing when a run succeeds, so the two lines that appear are vanadis's,
# through heimdallr's stdout. `--interval 1` because the default 30 would be 30 seconds of
# GIF spent waiting; the flag is real and this is what it is for.
#
# It fires once here, with the mode as it already is, which is the "it catches up" the README
# claims: a machine that was off across the transition is not left behind. Nothing changes
# colour, because nothing is out of date.
#
# Backgrounded, and that is not free. `&` makes a new process group, and an `--on-change`
# command with a child that touches the terminal gets SIGTTOU and stops the watcher without
# saying so. Nothing in this one does. See demo/README.md.
Type `heimdallr watch --interval 1 --on-change 'vanadis apply --variant "$1"' &` Enter

# Waited on and not slept through, because this is the step that failed silently once. A
# watcher stopped by SIGTTOU prints nothing and looks exactly like a working one, and the
# recording that came of it had a theme that never followed anything. `applied` is the
# start-up run's own output, so a tape that gets no theme change aborts here rather than
# producing a plausible GIF.
Wait+Screen@20s /papercolor-dark/
Sleep 2s

# A bare Enter before each command, and it earns its place twice.
#
# The watcher's output arrives while bash is already sitting at a prompt, so it lands on the
# prompt's own line and bash does not draw another one — the next command would be typed at
# a bare column zero, which reads like output rather than like something typed. Enter asks
# for the prompt back.
#
# It is also the only way the prompt is seen to follow the theme at all. starship re-reads
# its config on every prompt, so a prompt already on screen keeps the colours it was drawn
# with; the one after the flip is the one that shows the new palette.
Enter
Sleep 500ms

# Light. This is the Windows appearance setting, typed rather than clicked, and it changes
# the real desktop.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 1 /f >/dev/null` Enter

# The poll seeing the flip, waited on for the same reason as above. `papercolor-light` is on
# nothing that has been typed, so this matches the output or times out.
Wait+Screen@20s /papercolor-light/
Sleep 2s

Enter
Sleep 500ms

# And back, so the loop has no seam.
#
# This one is a `Sleep`. `papercolor-dark` has been on screen since the start-up run, so
# there is nothing here for `Wait+Screen` to match that has not matched already. The
# teardown checks this transition instead, through `vanadis current`, which knows which
# theme was applied last and not which words are on the screen.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d 0 /f >/dev/null` Enter
Sleep 4s

# The last frame is a prompt in the palette it started in.
Enter
Sleep 1500ms

Hide

# The second transition, which the screen could not distinguish. `vanadis current` prints the
# theme it applied last, so this is the check that the flip back fired rather than that the
# words were already there.
#
# `$?` and not the word itself, in every one of these: `Wait+Screen` matches the whole
# screen, and a command containing the string it waits for is matched by the typing rather
# than by the result.
Type `[ "$(vanadis current)" = "papercolor-dark" ]; echo back=$?` Enter
Wait+Screen@15s /back=0/

# The recorder's desktop, back where it was, and before anything is torn down: the shell is
# known good here, and this is the one step that has to happen. Killing the watcher first
# cost two recordings — the restore and its check ended up typed into a shell that had
# stopped reading, and the tape aborted on the check rather than on the thing that broke.
#
# `add` reporting 0 is the evidence. An earlier version read the value back with a second
# `reg.exe query`, and that query intermittently never returned: `restore=0` printed and then
# nothing, in two recordings out of three, with `grep -q` and with an `awk` that reads its
# input to the end alike. `add` has returned every time. The mechanism was not established,
# so the tape does not run the call that hangs and demo/README.md says to check the value
# afterwards.
Type `reg.exe add "$KEY" /v AppsUseLightTheme /t REG_DWORD /d $((ORIG)) /f >/dev/null 2>&1; echo restore=$?` Enter
Wait+Screen@15s /restore=0/

# And that is the whole teardown. Nothing is killed here.
#
# `kill %1` on the watcher was the last thing to wedge this tape: it signals the job's whole
# process group, and that group holds whatever `reg.exe` heimdallr had running at the time,
# so the shell stopped answering and `echo watcher=$?` never printed. Both background jobs
# get SIGHUP when vhs takes the shell away, and neither was left behind by the recording
# this GIF came from — checked with `pgrep -af "follow-terminal-bg|heimdallr watch"`
# afterwards, which is also how the disowned version was caught leaking.
