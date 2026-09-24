#!/usr/bin/env bash
# Play the game under tmux, and read the screen back — including its colours.
#
# How the game gets looked at without a person at the keyboard. `ORBS_DUMP` is a
# still photograph and throws colour away, so a fault in `orbs-tui/src/theme.rs`
# is invisible to it. This is the running game: real clock, real keystrokes, and
# `ink` to see what the terminal was told to draw.
#
#     scripts/tui.sh start                     # 120x45, the game's own grid
#     scripts/tui.sh start 177 38              # ...or any size
#     scripts/tui.sh type 'attend laboratory' 'kindle charcoal'
#     scripts/tui.sh type 'repeat 4' 'grind sage' 'end'   # literal — `end` is a word
#     scripts/tui.sh key Down Down Left        # arrows, for the maze
#     scripts/tui.sh key F5                    # function keys, by name
#     scripts/tui.sh wait 20                   # let the world run 20 ticks
#     scripts/tui.sh see                       # the screen, as text
#     scripts/tui.sh see -e                    # ...with the escape sequences
#     scripts/tui.sh ink 158 159               # ...decoded, glyph and colour
#     scripts/tui.sh stop
#
# Pace typing to the tick. `Sim::submit` queues a line for the *next* tick, so
# two lines sent inside one second land on the same tick and the second refuses.
# `type` sleeps a tick for you; `key` does not, `Sim::walk` spending no world
# time.
#
# Resizing a running session is fine and worth exercising: `tmux resize-window
# -t orbs -x 90 -y 24` reflows the screen, drops the rail into the border title,
# and draws the "too small" card below the 80x22 floor. See
# `blit::Screen::resize`.
set -euo pipefail

session="${ORBS_TMUX_SESSION:-orbs}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# A private socket, the way `play.sh` has one. This drove the *default* server
# and killed any session called `orbs`, destroying a developer's own with `||
# true` hiding the complaint. `-f /dev/null` is the other half: inheriting
# `~/.tmux.conf` made `see` and `ink` differ per machine.
tmux() { command tmux -L orbs-tui -f /dev/null "$@"; }

case "${1:-}" in
start)
  shift
  # 120x45 is `orbs_render::GRID`, so a capture is directly comparable with
  # `ORBS_DUMP`. 80x22 is the authoring floor; anything smaller draws the
  # "too small" card, which is a real screen rather than a refusal.
  cols="${1:-${ORBS_TMUX_COLS:-120}}"
  rows="${2:-${ORBS_TMUX_ROWS:-45}}"
  # Built first, and the binary run directly: `cargo run` prints its own
  # progress into the pane, on top of the game.
  cargo build -q -p orbs-tui --manifest-path "$root/Cargo.toml"
  tmux kill-session -t "$session" 2>/dev/null || true
  # Four defaults off, because this is the scripting tool: §4's boot is nine and
  # a half seconds, a sealed start (§11.5) has no `archive` to attend, a screen
  # part-way through leaving is not the one anyone asked about, and stopping at
  # the orb's menu puts a screen in front of every See-it line. Set any of them
  # back to watch the real thing — though `ORBS_PASSAGE=1` shows no crossing
  # here, this build holding a settled `Passing` it never advances (no CRT, so
  # no motion switch a player could reach). `cargo run -p orbs` for that.
  #
  # Passed rather than exported: tmux does not hand the client's environment to
  # a session on an already-running server, and it fails *silently*.
  tmux new-session -d -s "$session" -x "$cols" -y "$rows" \
    "$(printf 'ORBS_WIZARD=%q ORBS_SEED=%q ORBS_BOOT=%q ORBS_SEALED=%q ORBS_PASSAGE=%q ORBS_THRESHOLD=%q %q/target/debug/orbs-tui' \
      "${ORBS_WIZARD:-wizard}" "${ORBS_SEED:-181}" "${ORBS_BOOT-0}" "${ORBS_SEALED-0}" \
      "${ORBS_PASSAGE-0}" "${ORBS_THRESHOLD-0}" "$root")"
  sleep 1
  ;;
type)
  shift
  for line in "$@"; do
    # `-l`, and Enter on a call of its own. Without the literal flag tmux reads
    # the argument as a *key name* wherever one matches, and `end` closes every
    # `repeat` and `if` in the spell language. `-l` will not take a key name in
    # turn, which is why Enter cannot ride along.
    [ -n "$line" ] && tmux send-keys -t "$session" -l -- "$line"
    tmux send-keys -t "$session" Enter
    # One tick and a little, so each command gets its own.
    sleep 1.1
  done
  ;;
key)
  shift
  # Arrows and named keys: `key Down Down Left`, `key Escape`, `key F5`.
  # No sleep — `Sim::walk` is the third entry point and spends no world time,
  # so a player walks as fast as they can press.
  for k in "$@"; do
    tmux send-keys -t "$session" "$k"
    sleep 0.1
  done
  ;;
wait)
  # Let the world run. The clock is 1 Hz, so this is ticks *and* seconds — and
  # `meditate` is the in-game way to skip time without waiting for it.
  sleep "$((${2:-1}))"
  ;;
see)
  shift
  tmux capture-pane -t "$session" -p "$@"
  ;;
ink)
  shift
  # What the terminal was told to draw, cell by cell. `see` throws the colour
  # away and `see -e` hands back unreadable escapes, so this is the only way to
  # check `theme.rs` from outside.
  tmux capture-pane -t "$session" -p -e |
    "$root/scripts/ink.py" --cols "${1:-0},${2:-40}" --skip-blank "${@:3}"
  ;;
stop)
  tmux kill-session -t "$session" 2>/dev/null || true
  ;;
*)
  echo "usage: tui.sh {start [cols rows]|type <line>...|key <Key>...|wait [n]|see [-e]|ink [first last]|stop}" >&2
  exit 2
  ;;
esac
