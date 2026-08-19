#!/usr/bin/env bash
# Play the whole game, as a test suite.
#
# **The third layer.** `orbs-sim`'s tests prove the rules and `orbs-render`'s
# prove the picture; neither presses a key. This runs the real binary inside a
# real terminal, types at it, and reads the screen back — so it is the only thing
# in the project that exercises the event loop, the redraw diff, keyboard
# ownership between five surfaces, the clocks measured off `Time`, and colour as
# a terminal actually resolves it.
#
#     scripts/play.sh                    # all of it, ~1 minute
#     scripts/play.sh routing::          # one area
#     scripts/play.sh -- --test-threads 1    # one at a time, to watch
#
# **Not part of the gate.** The gate runs after every step and this takes a
# minute; every scenario is `#[ignore]`d so `cargo test --workspace` skips them.
# Its standing is `orbs-balance`'s instead: **run it after anything that touches
# the sim, the shell, or the loop.**
#
# Needs `tmux`. Without it every scenario prints SKIPPED and passes, which is
# deliberate — a missing terminal multiplexer is not a broken game — so read the
# output rather than the exit code when running somewhere new.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v tmux >/dev/null 2>&1; then
  echo "scripts/play.sh needs tmux; every scenario will skip without it" >&2
fi

# **A private socket, killed before and after.** The scenarios use one of their
# own so they cannot see the developer's sessions, their ~/.tmux.conf, or a
# `remain-on-exit` that would leave a window open after `quit` and hang the wait
# for it. A leftover server from an interrupted run is the one thing that can
# make a green suite go red, so it goes first.
tmux -L orbs-play kill-server 2>/dev/null || true
trap 'tmux -L orbs-play kill-server 2>/dev/null || true' EXIT

filter=""
if [ $# -gt 0 ] && [ "${1}" != "--" ]; then
  filter="$1"
  shift
fi
[ "${1:-}" = "--" ] && shift

exec cargo test --manifest-path "$root/Cargo.toml" \
  -p orbs-tui --test playing -- --ignored ${filter:+"$filter"} "$@"
