#!/usr/bin/env bash
# Play the tour (DESIGN.md §19, *the tour*): every surface, moving, one scene
# at a time, in the real window.
#
#     scripts/tour.sh                         # list the scenes
#     scripts/tour.sh all                     # every scene, one window
#     scripts/tour.sh forge                   # a group: every forge.*
#     scripts/tour.sh forge.lattice,lens      # names and groups
#     scripts/tour.sh forge.lattice           # one scene; yours when its steps end
#     scripts/tour.sh shots <dir> [what]      # screenshots and their frames as text, then exit
#
# In the window: F11 is the next scene, Shift+F11 the one before, and any other
# key takes the scene over. A debug build, since the scenes use the debug words.
# The scenes are crates/orbs-shell/src/tour/scenes.toml; ORBS_TOUR_FILE=<path>
# plays an edited copy without a rebuild.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Built first and run directly: `cargo run` would print its progress into the
# shots' stderr.
cargo build -q -p orbs --manifest-path "$root/Cargo.toml"
orbs="$root/target/debug/orbs"

# The switches that would outrank a scene, cleared, and the wizard pinned, as
# `dumps.sh` pins it, so a screenshot does not carry who took it. Both readers
# off, as `dumps.sh` has them: weights are a gitignored build artefact, and the
# tests that hold every scene to its claims run without them — with them,
# `purge s` asks a suggestion instead of the numbered question its scene shows.
clean=(env -u ORBS_PASSAGE -u ORBS_STRAIN -u ORBS_CRT -u ORBS_SIGHT -u ORBS_MOTION
  -u ORBS_AFTERGLOW -u ORBS_FIRE -u ORBS_SEED -u ORBS_BOOT -u ORBS_DUMP -u ORBS_CAPTURE
  -u ORBS_TOUR_SHOTS "ORBS_WIZARD=${ORBS_WIZARD:-wizard}"
  "ORBS_AUGURY=${ORBS_AUGURY:-off}" "ORBS_SCRIVENER=${ORBS_SCRIVENER:-off}")

case "${1:-list}" in
list)
  "${clean[@]}" ORBS_TOUR=list "$orbs"
  ;;
shots)
  out="${2:?usage: tour.sh shots <dir> [what]}"
  what="${3:-all}"
  mkdir -p "$out"
  out="$(cd "$out" && pwd)"
  # Only what a tour writes, so a directory with other files in it keeps them.
  rm -f "$out"/[0-9][0-9]-*.png "$out"/[0-9][0-9]-*.txt "$out/index.txt" "$out/stderr.log"
  # From the directory, so a stray F6 or F12 file lands there too. The timeout
  # is the backstop for a run that never reaches its last scene.
  rc=0
  (cd "$out" && "${clean[@]}" ORBS_TOUR="$what" ORBS_TOUR_SHOTS="$out" \
    timeout "${ORBS_TOUR_TIMEOUT:-900}" "$orbs" 2>"$out/stderr.log") || rc=$?
  # CLAUDE.md, *read the log*: the gate can be green while the GPU complains.
  grep -iE 'error|panic|warn' "$out/stderr.log" || true
  shots=0
  # `|| true`: `grep -c` fails when it counts none, and `set -e` would end here.
  [ -f "$out/index.txt" ] && shots=$(grep -c . "$out/index.txt" || true)
  echo "$shots screenshots in $out (index.txt lists them; each .png has its frame as .txt)"
  if [ "$rc" -ne 0 ]; then
    echo "the tour ended with $rc$([ "$rc" -eq 124 ] && echo ', timed out')" >&2
    exit "$rc"
  fi
  ;;
*)
  "${clean[@]}" ORBS_TOUR="$1" "$orbs"
  ;;
esac
