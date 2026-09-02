#!/usr/bin/env bash
# Capture every surface the game can draw, as text, into a directory.
#
# **The instrument for a refactor whose gate is that nothing changes.** Run it
# before the change and after, then `diff -r` the two directories: a byte for a
# byte, or the refactor moved something it was not asked to move.
#
# `ORBS_WIZARD` is pinned because the prompt name falls back to `$USER` and a
# baseline that varies with who ran it is not a baseline. Everything else is
# verbatim from CLAUDE.md's See-it blocks.
#
#     scripts/dumps.sh /tmp/before
#     ...make the change...
#     scripts/dumps.sh /tmp/after
#     diff -r /tmp/before /tmp/after
set -euo pipefail

out="${1:?usage: dumps.sh <output-directory>}"
mkdir -p "$out"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build -q -p orbs --manifest-path "$root/Cargo.toml"
orbs="$root/target/debug/orbs"

export ORBS_WIZARD=david
# **A live hazard, not belt-and-braces.** This runs from the repository root, so
# without the pin one captured screen could write `orbs-save.toml` and the next
# 55 would load it — a baseline that varies with what is lying in the directory
# is not a baseline. A dump ignores the file entirely unless `ORBS_SAVE` names
# one, and `off` says so out loud rather than relying on that default.
export ORBS_SAVE=off

# name=env-assignments...  — one capture per line, `%` separating name from env.
run() {
  local name="$1"; shift
  env "$@" "$orbs" > "$out/$name.txt" 2>"$out/$name.err" || true
  # A dump prints nothing to stderr in the ordinary case; keep the file only if
  # it has content, so `diff -r` is not full of empty noise.
  [ -s "$out/$name.err" ] || rm -f "$out/$name.err"
}

# --- boot ------------------------------------------------------------------
run boot_dark  ORBS_DUMP=1 ORBS_BOOT=dark
run boot_frame ORBS_DUMP=1 ORBS_BOOT=frame
run boot_post  ORBS_DUMP=1 ORBS_BOOT=post
# The card *finished*, which is the only place its two version lines appear —
# and the engine line is the one thing on it that differs between the frontends.
run boot_done  ORBS_DUMP=1 ORBS_BOOT=post:1

# --- the bare screen, and the two grids ------------------------------------
run bare        ORBS_DUMP=1
run bare_floor  ORBS_DUMP=1 ORBS_GRID=80x22
run line_typed  ORBS_DUMP=1 ORBS_LINE="grind sa"

# --- the laboratory, and every instrument animation ------------------------
run lab_rail   ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage"
run lab_flare  ORBS_BOOT=0 ORBS_FLARE=1 ORBS_DUMP="attend laboratory; kindle charcoal"
run lab_load   ORBS_BOOT=0 ORBS_LOAD=0.5 ORBS_DUMP="attend laboratory; move sage to mortar_and_pestle"
run lab_creep  ORBS_BOOT=0 ORBS_TICK=0.5 ORBS_DUMP="attend laboratory; grind sage; meditate 3"
run lab_bath   ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; meditate 6"
run lab_charged ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; meditate 9; empty mortar_and_pestle; move ground-sage to balneum_mariae"
run lab_flask  ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; meditate 14; siphon balneum_mariae; grind rock-salt; meditate 9; empty mortar_and_pestle; mix sage-tincture with ground-salt; meditate 5"
run lab_tint   ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to mortar_and_pestle"
run lab_husks  ORBS_BOOT=0 ORBS_DUMP="attend laboratory; grind sage; meditate 9; move ground-sage to dispensary"
run lab_clarity ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; meditate 14; grind rock-salt; meditate 9; empty mortar_and_pestle; mix sage-tincture with ground-salt; meditate 12; distil clarified-draught; meditate 60; status"

# --- the fault latch -------------------------------------------------------
run fault_latched ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
  ORBS_EDIT=$'edit\nrepeat 5\nwield zzz\nend\n<esc>\nquit' \
  ORBS_THEN="invoke broken; meditate 20; attend archive"

# --- the lens --------------------------------------------------------------
run lens_ward  ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; dial second borax; probe; survey prism; survey second"
run lens_twice ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; dial first alum; probe; survey first; survey second"
run lens_spill ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; debug_ward; probe; peruse lens.log"

# --- the archive -----------------------------------------------------------
run maze        ORBS_BOOT=0 ORBS_DUMP="attend archive; research"
run maze_walked ORBS_BOOT=0 ORBS_DUMP="attend archive; research; follow east; follow east"
run maze_wander ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" ORBS_WALK=$'<right>\n<right>\n<down>\n<down>\n<left>'
run maze_seed3  ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research; wander"
run maze_seed11 ORBS_SEED=11 ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research; wander"
run maze_glean  ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive; research; debug_spawn gleaning-scroll; wield gleaning-scroll; wander"
run maze_marks  ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research; follow south; follow south; follow north; survey south"
run maze_log    ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; research; follow west; follow west; peruse archive.log"

# --- the weave screen ------------------------------------------------------
run weave       ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave"
run weave_aim   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'mastery\n<down>\ntake'
run weave_early ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'<down>\n<right>'

# --- the editor ------------------------------------------------------------
run editor_write ORBS_DUMP="attend laboratory; scribe brewing" \
  ORBS_EDIT=$'edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit' \
  ORBS_THEN="invoke brewing; meditate 40"
run editor_verbatim ORBS_DUMP="attend laboratory; scribe morning" \
  ORBS_EDIT=$'edit\nmake a potion of clarity\n<esc>\nquit' ORBS_THEN="peruse morning.spell"
run editor_interpret ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe check" \
  ORBS_EDIT=$'edit\nmake a potion of clarity\nif the mortr is bare\nsurvey\nend\nxyzzy plugh\n<esc>\ninterpret'
run editor_count ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive; scribe check" \
  ORBS_EDIT=$'edit\nif the cabinet has 4 fragment\nwield lectern\nend\n<esc>\ninterpret'

# --- the manual, in every room --------------------------------------------
for room in laboratory archive lens sanctum menagerie grimoire forge arsenal tower; do
  run "help_$room" ORBS_BOOT=0 ORBS_DUMP="attend $room; help"
done
run help_floor    ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend lens; help"
run recall_pages  ORBS_BOOT=0 ORBS_DUMP="recall clarity; recall gleaning-scroll"
run recall_words  ORBS_BOOT=0 ORBS_DUMP="recall repeat; recall until; recall marks"
run recall_script_arch ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; recall scripting"
run recall_script_lab  ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend laboratory; recall scripting"

# --- the tester's tools ----------------------------------------------------
run spawn_bare  ORBS_BOOT=0 ORBS_DUMP="debug_spawn"
run spawn_homes ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 4; debug_spawn clarity; debug_spawn sage; survey cabinet; survey arsenal"
run spawn_places ORBS_BOOT=0 ORBS_DUMP="debug_spawn fragment 4 lectern; debug_spawn clarity 1 arsenal; debug_spawn sage 1 arsenal; debug_spawn sage 1 north; debug_spawn sage 1 laboratory"
run swap        ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_swap; survey dispensary; verify dispensary; verify laboratory"
run dev_shelf   ORBS_BOOT=0 ORBS_DUMP="survey grimoire"
run dev_list    ORBS_BOOT=0 ORBS_DUMP="debug_spell"

# --- the arsenal -----------------------------------------------------------
run arsenal ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught; distil clarified-draught; meditate 60; empty alembic; move clarity to arsenal; attend archive; survey arsenal"
run arsenal_door ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to arsenal"

# --- scrolls ---------------------------------------------------------------
run scroll_quick   ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn quickening-scroll; wield quickening-scroll; grind sage; meditate 4"
run scroll_verdant ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn verdant-scroll 4; wield verdant-scroll; wield verdant-scroll; wield verdant-scroll; wield verdant-scroll; survey dispensary"

# --- the sanctum -----------------------------------------------------------
run pylon_board  ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; haul wellspring barrier; haul wellspring conduit; haul barrier conduit"
run pylon_refuse ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; haul wellspring barrier; haul wellspring barrier; haul wellspring wellspring; haul conduit barrier"
run pylon_worn   ORBS_BOOT=0 ORBS_DUMP="attend sanctum; survey pylon; meditate 3600; survey pylon; muster; survey pylon"
run pylon_done   ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; debug_course; haul conduit barrier; survey pylon; status"

# --- the menagerie ---------------------------------------------------------
# **Three ticks of one approach**, because the board is the only surface in the
# game that moves between landings and a single capture cannot show that. It
# drew four identical frames for a whole approach until `Figure` carried `until`.
run chant_board   ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon"
run chant_rising  ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; meditate 2"
run chant_refuse  ORBS_BOOT=0 ORBS_DUMP="attend menagerie; sing skyward; summon; summon; sing nothing"
run chant_keys    ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; chorus" ORBS_CHANT=$'<up>\n<left>'
run chant_patient ORBS_BOOT=0 ORBS_PATIENT=1 ORBS_DUMP="attend menagerie; summon; chorus" ORBS_CHANT=$'<up>\n<left>'
run chant_collapse ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; meditate 18; survey pylon"
run chant_troop   ORBS_BOOT=0 ORBS_DUMP="recall troop; debug_spawn troop 3; survey arsenal"
run recall_script_sanctum ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend sanctum; recall scripting"

# --- the bailey ------------------------------------------------------------
# **The whole domain was absent from this file**, which shipped with Phase 8 and
# went unnoticed until quintessence changed the board and the diff came back
# clean. A capture set that does not hold a surface cannot tell you when it moves,
# and the rampart is the most numerically dense picture in the game.
run siege_board    ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend"
run siege_pledged  ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; pledge d6 succour; survey coffer"
# The refusal, which is the decision the resource exists to force: three full
# rounds are exactly the pool, so the fourth has nothing to spend.
run siege_short    ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; pledge d8 line; pledge d6 succour; hold; pledge d20 buckler; pledge d8 line; pledge d6 succour; hold; pledge d20 buckler; pledge d8 line; pledge d6 succour; hold; pledge d20 buckler"
# A worn tower opens with half a pool — §19's integrity coupling, on screen.
run siege_worn     ORBS_BOOT=0 ORBS_DUMP="attend sanctum; meditate 3600; attend bailey; defend; survey coffer"
run siege_readings ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; survey enemy; survey garrison; survey d20"
run siege_round    ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; hold; peruse bailey.log"
run siege_refuse   ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; pledge d20 line; pledge buckler d20; deploy sage"
run siege_over     ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; debug_siege; hold; survey rampart"
run recall_script_bailey ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend bailey; recall scripting"

# --- the forge -------------------------------------------------------------
# **Captured from the day it shipped**, which the bailey was not: Phase 8 went a
# whole phase without a block here and nothing in the repository could have told
# you when its board moved, because a diff over a surface this file does not hold
# comes back clean. The blindness looks exactly like stability.
#
# The lattice is the densest *shape* in the game where the rampart is the densest
# numbers, so what these need to catch is a glyph in the wrong column.
run forge_empty    ORBS_BOOT=0 ORBS_DUMP="attend forge"
run forge_open     ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried"
# One snap, so the grid and the residue disagree — which is the state a player
# spends the whole puzzle in and the one a board can most easily draw wrong.
run forge_snapped  ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; snap belt"
run forge_bound    ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; snap apex; anneal; survey mortar_and_pestle"
# The refusals, and the one that prices the domain: a charm you cannot pay for.
run forge_refuse   ORBS_BOOT=0 ORBS_DUMP="attend forge; snap apex; anneal; imbue sage hurried; imbue mortar_and_pestle nonsense"
run forge_readings ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; survey apex; survey belt; survey hem; survey hurried"
run recall_script_forge ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend forge; recall scripting"

# The apprentice's guide. **One per room**, because the worked example is built
# from where you are standing — five rooms with lines of their own, and the
# grimoire borrowing the laboratory's and saying so.
for room in laboratory archive lens sanctum menagerie grimoire forge; do
  run "apprentice_$room" ORBS_BOOT=0 ORBS_DUMP="attend $room; recall apprentice"
done

# --- the satchel, and the second cursor (§8's channel) ----------------------
#
# **`debug_take` on every line but the first.** `satchel_1` opens at 24
# experience and `cursors_1` at 40, so an honest road to these screens is two
# hundred ticks of the laboratory before the thing being captured appears — the
# setup cost `debug_spawn` already exists to skip.
run satchel_gated  ORBS_BOOT=0 ORBS_DUMP="attend menagerie; queue skyward"
run satchel_queued ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1; \
  queue skyward; queue earthward; queue skyward; survey satchel"
run satchel_bare   ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1; survey satchel"
run satchel_take   ORBS_BOOT=0 ORBS_DUMP="debug_take; debug_take tbi_b; debug_take cursors_1"
run satchel_drain  ORBS_BOOT=0 ORBS_GRID=110x40 \
  ORBS_DUMP="attend menagerie; debug_take satchel_1; queue skyward; queue earthward; scribe drain" \
  ORBS_EDIT=$'edit\nrepeat 2\npull note from satchel\nsurvey note\nend\n<esc>\nquit' \
  ORBS_THEN="invoke drain; meditate 10; survey satchel"
run satchel_fork   ORBS_BOOT=0 ORBS_GRID=110x40 \
  ORBS_DUMP="attend menagerie; debug_take satchel_1; debug_take cursors_1; scribe both" \
  ORBS_EDIT=$'edit\npart filling()\nqueue skyward\nqueue earthward\nend\nalongside filling()\nrepeat 2\npull note from satchel\nsurvey note\nend\n<esc>\nquit' \
  ORBS_THEN="invoke both; meditate 12; peruse menagerie.log"
run recall_satchel ORBS_BOOT=0 ORBS_DUMP="recall queue; recall pull; recall alongside"

# The two shipped worked examples, and the rail counting what they run.
run satchel_two_spells ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_take satchel_1" \
  ORBS_THEN="invoke milling; meditate 80; peruse laboratory.log"
run satchel_two_cursors ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1; debug_take cursors_1" \
  ORBS_THEN="invoke coursing; meditate 400; peruse sanctum.log"
run satchel_rail ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1; debug_take cursors_1" \
  ORBS_THEN="invoke coursing; meditate 6; status"

# --- bindings --------------------------------------------------------------
run bind_invoke ORBS_BOOT=0 ORBS_DUMP="attend laboratory; invoke first_light; attend archive; meditate 6"
run bind_holding ORBS_BOOT=0 ORBS_DUMP="attend sanctum; invoke holding; meditate 400" ORBS_THEN="peruse sanctum.log"

echo "captured $(ls -1 "$out"/*.txt | wc -l) screens into $out"
