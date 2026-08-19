#!/usr/bin/env bash
# Play the game under tmux, and read the screen back — including its colours.
#
# **This is how the game gets looked at without a person at the keyboard.**
# `ORBS_DUMP` is a still photograph and needs an environment variable per thing
# it cannot pose; it also throws colour away entirely, so a fault in
# `orbs-tui/src/theme.rs` is invisible to it. This is the running game: real
# clock, real keystrokes, real surfaces, and `ink` to see what the terminal was
# actually told to draw.
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
# **Pace typing to the tick.** `Sim::submit` queues a line for the *next* tick,
# so two lines sent inside one second both land on the same tick with no time
# passing between them — a `grind` and the `empty` that was meant to precede it
# arrive together and the second refuses. `type` sleeps a tick for you; `key`
# does not, because `Sim::walk` spends no world time at all.
#
# **Resizing a running session is fine**, and worth exercising: `tmux
# resize-window -t orbs -x 90 -y 24` reflows the whole screen, drops the rail
# into the border title when the columns run short, and draws the "too small"
# card below the 80x22 floor. It did *not* used to be fine — see
# `blit::Screen::resize`.
set -euo pipefail

session="${ORBS_TMUX_SESSION:-orbs}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "${1:-}" in
start)
  shift
  # 120x45 is `orbs_render::GRID`, so a capture is directly comparable with
  # `ORBS_DUMP`. 80x22 is the authoring floor; anything smaller draws the
  # "too small" card, which is a real screen rather than a refusal.
  cols="${1:-${ORBS_TMUX_COLS:-120}}"
  rows="${2:-${ORBS_TMUX_ROWS:-45}}"
  # **Built first, and the binary run directly.** `cargo run` prints its own
  # progress into the pane, which lands on top of the game.
  cargo build -q -p orbs-tui --manifest-path "$root/Cargo.toml"
  tmux kill-session -t "$session" 2>/dev/null || true
  # **`ORBS_BOOT=0` by default**, because this is the scripting tool and §4's
  # sequence is nine and a half seconds. That is the *development* half of the
  # instant-startup argument, and the switch is where it belongs — set
  # `ORBS_BOOT=` to watch the game open properly instead.
  tmux new-session -d -s "$session" -x "$cols" -y "$rows" \
    "$(printf 'ORBS_WIZARD=%q ORBS_SEED=%q ORBS_BOOT=%q %q/target/debug/orbs-tui' \
      "${ORBS_WIZARD:-wizard}" "${ORBS_SEED:-181}" "${ORBS_BOOT-0}" "$root")"
  sleep 1
  ;;
type)
  shift
  for line in "$@"; do
    # **`-l`, and Enter on a call of its own.** Without the literal flag tmux
    # reads the argument as a *key name* wherever one matches: `end` becomes the
    # End key, `up` an arrow, `home` Home. `end` closes every `repeat` and every
    # `if` in the spell language, so typing a spell without this sent a cursor
    # key where the word belonged and the block never closed. `-l` will not take
    # a key name in turn, which is why Enter cannot ride along.
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
  # **What the terminal was actually told to draw**, cell by cell. `see` throws
  # the colour away and `see -e` hands back escapes nobody can read down a
  # column, so this is the only way to check `theme.rs` from outside.
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
