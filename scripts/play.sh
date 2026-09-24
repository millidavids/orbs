#!/usr/bin/env bash
# Play the whole game, as a test suite.
#
# The third layer. `orbs-sim`'s tests prove the rules and `orbs-render`'s prove
# the picture; neither presses a key. This runs the real binary in a real
# terminal, so it is the only thing that exercises the event loop, the redraw
# diff, keyboard ownership between five surfaces, the clocks measured off
# `Time`, and colour as a terminal resolves it.
#
#     scripts/play.sh                    # all of it, ~1 minute
#     scripts/play.sh routing::          # one area
#     scripts/play.sh -- --test-threads 1    # one at a time, to watch
#
# Not part of the gate — the gate runs after every step and this takes a minute,
# so every scenario is `#[ignore]`d. Its standing is `orbs-balance`'s instead:
# run it after anything that touches the sim, the shell, or the loop.
#
# Needs `tmux`. Without it every scenario prints SKIPPED and passes, which is
# deliberate — a missing terminal multiplexer is not a broken game — so read the
# output rather than the exit code when running somewhere new.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v tmux >/dev/null 2>&1; then
  echo "scripts/play.sh needs tmux; every scenario will skip without it" >&2
fi

# A private socket, killed before and after, so the scenarios cannot see the
# developer's sessions, their ~/.tmux.conf, or a `remain-on-exit` that hangs the
# wait after `quit`. A leftover server from an interrupted run is the one thing
# that can make a green suite go red, so it goes first.
tmux -L orbs-play kill-server 2>/dev/null || true
# And the scratch directories a killed run left behind — `Game::drop` removes
# its own, but ~100 per run accumulate fast after a hard kill.
#
# `${TMPDIR:-/tmp}` because the harness uses `std::env::temp_dir()`: a hardcoded
# `/tmp` swept nothing wherever `TMPDIR` is set. Only what is more than an hour
# old, because the glob is not scoped to this process and an unconditional sweep
# deletes a *concurrent* run's live scratch out from under it.
find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'play-*' -type d -mmin +60 \
  -exec rm -rf {} + 2>/dev/null || true
# INT and TERM as well as EXIT: an interrupted run should tear its server down
# rather than leave one for the next run to trip over.
trap 'tmux -L orbs-play kill-server 2>/dev/null || true' EXIT INT TERM

# A trap cannot survive `SIGKILL`, and neither can `Game`'s `Drop`: six killed
# runs once leaked 573 `orbs-tui` processes and took a 32-core machine to a load
# average of 581.
#
# So the backstop is in the game, not here — nothing a parent writes runs after
# it is killed. `orbs-tui` exits when its terminal goes away (`watch_for_hangup`
# in `orbs-tui/src/drive.rs`). What a hard kill leaves is a stale socket, which
# the line above clears on the next run.

filter=""
if [ $# -gt 0 ] && [ "${1}" != "--" ]; then
  filter="$1"
  shift
fi
[ "${1:-}" = "--" ] && shift

exec cargo test --manifest-path "$root/Cargo.toml" \
  -p orbs-tui --test playing -- --ignored ${filter:+"$filter"} "$@"
