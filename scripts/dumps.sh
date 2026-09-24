#!/usr/bin/env bash
# Capture every surface the game can draw, as text, into a directory.
#
# The instrument for a refactor whose gate is that nothing changes: run it
# before and after, then `diff -r` the two directories.
#
# `ORBS_WIZARD` is pinned because the prompt name falls back to `$USER`, and a
# baseline that varies with who ran it is not a baseline.
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
# This runs from the repository root, so without the pin one capture could write
# `orbs-save.toml` and the next 55 would load it. Said out loud rather than left
# to the default.
export ORBS_SAVE=off

# name=env-assignments...  — one capture per line, `%` separating name from env.
run() {
  local name="$1"; shift
  # Both readers off first: weights are a gitignored build artefact, so a
  # capture made against them is not reproducible from a clean checkout. A
  # block wanting one names it in its own arguments, which come after and win.
  env ORBS_AUGURY=off ORBS_SCRIVENER=off "$@" "$orbs" > "$out/$name.txt" 2>"$out/$name.err" || true
  # A dump prints nothing to stderr in the ordinary case; keep the file only if
  # it has content, so `diff -r` is not full of empty noise.
  [ -s "$out/$name.err" ] || rm -f "$out/$name.err"
}

# --- boot ------------------------------------------------------------------
run boot_dark  ORBS_DUMP=1 ORBS_BOOT=dark
run boot_frame ORBS_DUMP=1 ORBS_BOOT=frame
run boot_post  ORBS_DUMP=1 ORBS_BOOT=post
# The name arriving a letter at a time. One capture per letter, because what has
# to hold is the order — a single frame cannot show that the letter in flight is
# the only one moving.
run boot_o     ORBS_DUMP=1 ORBS_BOOT=post:0.04
run boot_r     ORBS_DUMP=1 ORBS_BOOT=post:0.09
run boot_b     ORBS_DUMP=1 ORBS_BOOT=post:0.14
run boot_s     ORBS_DUMP=1 ORBS_BOOT=post:0.19
# ...then what the name stands for, on its own clock and after the letters.
run boot_words ORBS_DUMP=1 ORBS_BOOT=post:0.24
run boot_said  ORBS_DUMP=1 ORBS_BOOT=post:0.31
# ...and the card leaving the same way. Watch the box: it stays while its
# contents collapse, because it is the pane the game arrives in.
run boot_close ORBS_DUMP=1 ORBS_BOOT=close:0.4
run boot_gone  ORBS_DUMP=1 ORBS_BOOT=close:1.0
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

# --- the scrivener ---------------------------------------------------------
# A loose spell, and the two views of it. These lines are ones the deterministic
# pipeline cannot read, so the capture exercises the reader rather than the
# parser; `interpret` marks what it read with `≈`, which makes a misreading
# visible. `stub` rather than a trained reader, per the header.
run scrivener_read ORBS_BOOT=0 ORBS_SCRIVENER=stub ORBS_DUMP="attend laboratory; scribe loose" \
  ORBS_EDIT=$'edit\nwork the sage down\nhang on ten ticks\n<esc>\ninterpret'
# ...and the same file with no reader: lines untouched, no `≈`. The pair is the
# point — one capture cannot show that a reading changed something.
run scrivener_off  ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe loose" \
  ORBS_EDIT=$'edit\nwork the sage down\nhang on ten ticks\n<esc>\ninterpret'

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
run weave_aim   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'ley\n<right>\n<down>\ntake'
run weave_early ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'<down>\n<right>'
run weave_lines ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'mastery\n<down>\n<right>\ntake'
run weave_reach ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; debug_reach laboratory_2; weave" ORBS_WEAVE=$'mastery'

# --- the weave at a longer game -----------------------------------------------
# Six-figure thresholds. Before `figures()` the last two totals printed
# `121766250000`, and only a capture like this would have shown it.
run weave_long   ORBS_LENGTH=long ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave"
run weave_medium ORBS_LENGTH=medium ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave"

# --- the orb's menu -----------------------------------------------------------
# `menu` opens it; `quit` leaves, asking first. Without these the instrument is
# blind to the surface, and that blindness looks exactly like stability.
run menu         ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu"
run menu_unknown ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="zorb"
run menu_resume  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="resume"
run menu_narrow  ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="menu"
# The play page: the towers and `new` on one screen. They were two words a level
# up, which asked a player to tell *open one you have* from *raise one* before
# anything had said there was a difference.
run menu_play    ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="play"
run menu_lengths ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='play\nnew'
# The mark on the chosen row is the point of the capture: a settings screen that
# lists what you *could* pick without saying what is picked is a list of things
# you might already have done.
run menu_settings ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="settings"
# Each page says what every setting is set to. `ORBS_SAVE=off` is exported
# above, so these are a first launch's values and cannot drift with whatever is
# in a real profile.
run menu_sound   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\nsound'
run menu_tube    ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\ntube'
run menu_access  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\naccess'
run menu_habits  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\nhabits'
# A word steps a setting; a word and a value names one. Both reach the same
# place, which is why neither needs a page of its own.
run menu_crt_off ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\ntube\ncrt off'
# The one page whose rows are a padded column. `menu_setting_row` writes
# `[value]` straight after the row's text, so a row one cell short puts one
# bracket out of line down the whole page — it happened.
# `every_settings_row_is_padded_to_the_same_width` measures it; this shows it.
run menu_volumes ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\nsound\nhum\nvoice quiet'
# ...and the choice made, which steps back to the page it was made on. A dump
# keeps nothing, so it cannot show the choice being remembered.
#
# This said `settings\nplain` for one version and quietly captured a refusal: a
# See-it line that still runs is not one that still sees the thing.
run menu_plain   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\nhabits\nreading plain'
# §6 on a settings page: a word it does not know is named, with the page still
# there to choose from. Kept as its own capture rather than as a side effect of
# a stale one.
run menu_unsettable ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU='settings\nzorb'
# ...and the question `quit` asks, which is the whole of what it now does before
# the second one.
run quit_asks    ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="quit"
run quit_again   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="quit" ORBS_THEN="quit"
run quit_off     ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="quit" ORBS_THEN="status; quit"

# --- the threshold ------------------------------------------------------------
# The screen a player sees first, between the boot card and any tower.
# `ORBS_THRESHOLD=1` opens it; nothing types a word there, having no prompt to.
#
# These are the smaller half of the gate: a dump builds no `App`, so the clock
# gate and the keyboard routing do not exist in one and what this pins is the
# picture. The behaviour is held by `sim::plugin`, `shell::plugin` and
# `the_threshold_has_no_way_to_close_the_menu`.
run threshold        ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1
# `resume` is not on it and not answered by it — there is no tower behind this
# one. The complaint is the capture: the word is refused, not silently ignored.
run threshold_resume ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="resume"
# `new` reaches the lengths with `ORBS_SAVE=off`. It used to refuse, which made
# the threshold unreachable-past from this instrument, from the played-game
# suite, and from any session run with saving off.
run threshold_new    ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU='play\nnew'
# ...and the play page says what is true, which is that there are none to open
# yet — with `new` under it, which is the whole reason the two share a page.
run threshold_play   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="play"
# `abandon` on an empty slot, which is all `ORBS_SAVE=off` can reach. What this
# pins is the refusal: §6 forbids a bare error, and this is the one word that
# must never act on a slot it did not find.
#
# The question and the answer need towers on disk. Pointing `ORBS_SAVE` at a
# directory would make the capture depend on what the last run left in it — the
# varying baseline the header refuses — so those halves are unit tests instead.
run threshold_abandon ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU='play\nabandon 1'

# --- the manual --------------------------------------------------------------
# The real painter on the real book, rather than the `screens` example's
# hand-built replica — a replica would be a second opinion about a layout the
# dump can simply draw.
#
# What is at risk in a manual is the wrapping: a paragraph, a title, a scroll
# hint and an input line all fitting, at the floor as well as at the grid.
run manual         ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual"
run manual_chapter ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="orbs"
# At the floor, where a chapter is taller than the pane and says so.
run manual_narrow  ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="keys"
# A chapter it does not have. §6: never a bare error.
run manual_unknown ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="zorb"
# The contents at the floor, where it used to silently lose five chapters:
# "one screen by construction" stopped being true the moment a sixteenth was
# added. It pages now.
run manual_contents_narrow ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual"
# The three chapters the manual builds rather than authors, out of the keys
# `recall` already holds. A verb authored tomorrow appears in the first with no
# code change, so what this pins is the *column*, not the content.
run manual_commands ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="commands"
run manual_language ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="language"
run manual_inside   ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="inside"
# The licences, which the BSD-2-Clause notice has to reach a player through.
run manual_notices  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="notices"
# ...and out, one level at a time: a chapter to the contents, the contents to
# the menu it opened over.
run manual_back    ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL='keys\n<esc>'
run manual_closed  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL='keys\n<esc>\n<esc>'
# The floor. One word shorter than the in-tower page, so if that fits so does
# this — pinned anyway, for the day the top page grows and nobody checks.
run threshold_narrow ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_THRESHOLD=1 ORBS_DUMP=1

# --- a sealed tower -----------------------------------------------------------
run sealed_doors ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend archive; attend stacks; survey archive; survey /tower"
run sealed_rail  ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory; attend archive"
run sealed_opens ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught 1; distil clarified-draught; meditate 60; attend archive"
run sealed_wall  ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach laboratory_3; attend bailey; debug_reach sanctum_1; attend bailey"

# --- the ley line's grants ----------------------------------------------------
run ley_line     ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE=$'ley\n<right>\n<right>\n<down>'
run grant_fuel   ORBS_BOOT=0 ORBS_DUMP="debug_take fuel_1; attend laboratory; kindle charcoal"
run grant_pool   ORBS_BOOT=0 ORBS_DUMP="status; debug_take pool_1; status"
run grant_thrift ORBS_BOOT=0 ORBS_DUMP="debug_take thrift_1; attend forge; imbue mortar_and_pestle hurried"
run road_lab     ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory; debug_reach laboratory_1"
run road_recall  ORBS_BOOT=0 ORBS_DUMP="attend laboratory; recall"

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

# A verb whose argument explained nothing asks rather than running: `verify
# gibberish` used to audit the whole tower with the player's word discarded. The
# last three lines are the ones that must *not* change — `light athanor`,
# `look around` and bare `survey` all still run. `status gibberish` is the same
# refusal for a verb that takes nothing: it names the words it could not use.
run reading_asks  ORBS_BOOT=0 ORBS_AUGURY=off ORBS_DUMP="attend laboratory; verify gibberish; status gibberish; digest husks; move charcoal to athanor; light athanor; look around; survey"
# ...and the same lines with a reader standing by, which is the shipping default
# and what the `off` capture cannot speak for. The grammar rather than the
# trained model, per the header.
run reading_asks_read ORBS_BOOT=0 ORBS_AUGURY=grammar ORBS_DUMP="attend laboratory; verify gibberish; digest husks; look around; survey"

# --- the augury ------------------------------------------------------------
# Both halves, and the pair is the point. `off` is these lines reaching `!` as
# they did before the augury existed; `stub` is the same lines read, echoed `≈`,
# and run. A change that routed a literal command through a reader would move
# the first file and nothing else would notice.
#
# `off` is written out even though `run` supplies it, because this is the block
# whose subject is the reader being absent.
run augury_off  ORBS_BOOT=0 ORBS_AUGURY=off ORBS_DUMP="attend laboratory; turn the sage into powder; smash the sage; what is in here"
run augury_stub ORBS_BOOT=0 ORBS_AUGURY=stub ORBS_DUMP="attend laboratory; turn the sage into powder; smash the sage; what is in here"
# A command typed properly, with a reader standing by. It must read identically
# to the same dump without one — the reader is never consulted for these.
run augury_typed ORBS_BOOT=0 ORBS_AUGURY=stub ORBS_DUMP="attend laboratory; grind sage; survey; recall brewing"
# ...and the reader's answer refused, because `grind` is the mortar's word and
# nobody is standing at the mortar. The echo still shows what it heard.
run augury_elsewhere ORBS_BOOT=0 ORBS_AUGURY=stub ORBS_DUMP="turn the sage into powder"
# The grammar, reading the authored templates rather than a fixed table.
# `work the sage down` is a `say` template the matcher cannot read; `crush the
# sage` is already a `grind` synonym and would prove nothing.
run augury_grammar ORBS_BOOT=0 ORBS_AUGURY=grammar ORBS_DUMP="attend laboratory; work the sage down; let me read the feed.log"

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

run recall_script_sanctum ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend sanctum; recall scripting"

# --- the menagerie ---------------------------------------------------------
# A beast arriving, called in wrong, and held. The answer row and its marks are
# the half of the board a painter can get wrong — a mark one column off its
# number sends a player to the wrong row — and only a *called* circle draws
# them. The default seed's first beast has two turned wires, so `circle_open`
# also captures the `~` mark.
run circle_open     ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon"
run circle_balk     ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; limn keystone heed; limn sunwise oppose; summon"
run circle_held     ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; debug_circle; summon; survey arsenal"
run circle_refuse   ORBS_BOOT=0 ORBS_DUMP="attend menagerie; limn keystone heed; summon; limn keystone sunwise; stop circle"
run circle_readings ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; limn keystone heed; survey keystone; survey circle"
run circle_troop    ORBS_BOOT=0 ORBS_DUMP="recall troop; debug_spawn troop 3; survey arsenal"
# A sealed tower's first beast: the lesser circle, a dark glyph refused, and a
# call that balks — the board with one line and four columns in the same footprint.
run circle_lesser   ORBS_SEALED=1 ORBS_BOOT=0 \
  ORBS_DUMP="debug_reach lens_1; attend menagerie; summon; limn sunwise heed; limn keystone mirror; summon"
# The search that knows the logic, through its log — a spell's records never reach the pane.
run circle_winnow   ORBS_BOOT=0 ORBS_DUMP="attend menagerie; invoke winnowing" \
  ORBS_THEN="meditate 400; peruse menagerie.log"
run recall_circle   ORBS_BOOT=0 ORBS_DUMP="recall yoke; recall oppose; recall sunwise; recall keystone; recall fervour; recall limn"
run recall_script_menagerie ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend menagerie; recall scripting"

# --- the bailey ------------------------------------------------------------
# The whole domain was absent from this file until quintessence changed the
# board and the diff came back clean. The rampart is the most numerically dense
# picture in the game.
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
# Captured from the day it shipped, which the bailey was not.
#
# The lattice is the densest *shape* in the game where the rampart is the
# densest numbers, so what these catch is a glyph in the wrong column.
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

# The apprentice's guide, one per room: the worked example is built from where
# you are standing, and the grimoire borrows the laboratory's and says so.
for room in laboratory archive lens sanctum menagerie grimoire forge; do
  run "apprentice_$room" ORBS_BOOT=0 ORBS_DUMP="attend $room; recall apprentice"
done

# --- the satchel, and the second cursor (§8's channel) ----------------------
#
# `debug_take` on every line but the first: `satchel_1` opens at 24 experience
# and `cursors_1` at 40, so an honest road here is two hundred ticks of setup
# before the thing being captured appears.
run satchel_gated  ORBS_BOOT=0 ORBS_DUMP="attend menagerie; queue heed"
run satchel_queued ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1; \
  queue heed; queue yoke; queue heed; survey satchel"
run satchel_bare   ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1; survey satchel"
run satchel_take   ORBS_BOOT=0 ORBS_DUMP="debug_take; debug_take tbi_b; debug_take cursors_1"
run satchel_drain  ORBS_BOOT=0 ORBS_GRID=110x40 \
  ORBS_DUMP="attend menagerie; debug_take satchel_1; queue heed; queue yoke; scribe drain" \
  ORBS_EDIT=$'edit\nrepeat 2\npull note from satchel\nsurvey note\nend\n<esc>\nquit' \
  ORBS_THEN="invoke drain; meditate 10; survey satchel"
run satchel_fork   ORBS_BOOT=0 ORBS_GRID=110x40 \
  ORBS_DUMP="attend menagerie; debug_take satchel_1; debug_take cursors_1; scribe both" \
  ORBS_EDIT=$'edit\npart filling()\nqueue heed\nqueue yoke\nend\nalongside filling()\nrepeat 2\npull note from satchel\nsurvey note\nend\n<esc>\nquit' \
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

# --- the passage -----------------------------------------------------------
# A crossing is an edge and a dump observes no edges, so `ORBS_PASSAGE_AT` holds
# the last command back and poses one frame of the screen leaving. Everything
# else in this file must be unchanged by the feature existing — a settled
# `Passing` is a no-op, and that is the gate.
#
# Three fractions, because one frame of a motion says nothing about its shape:
# the wake at 0.15, the empty beat at 0.50, the new screen arriving at 0.85.
#
# Read these for the two regions going different ways: the top strip leaves
# upward, the side block rightward, and the transcript between them is
# untouched — the whole argument for crossing parts rather than the pane.
run cross_out    ORBS_BOOT=0 ORBS_PASSAGE_AT=0.15 ORBS_DUMP="attend laboratory; attend forge"
run cross_beat   ORBS_BOOT=0 ORBS_PASSAGE_AT=0.50 ORBS_DUMP="attend laboratory; attend forge"
run cross_in     ORBS_BOOT=0 ORBS_PASSAGE_AT=0.85 ORBS_DUMP="attend laboratory; attend forge"
run cross_still  ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend laboratory; grind sage; attend archive"
# A panel leaving for a room that has none. The block is sized from the union of
# both screens' regions; sized from the arriving one alone, the laboratory's
# instruments would cut instead of going.
run cross_union  ORBS_BOOT=0 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend forge; attend laboratory; attend forge"
# The L, and the capture that would have caught it wiping the transcript. At
# this grid `Along::of` puts the panel across the top while the maze claims
# columns from the right, so what the transcript gives up is not a rectangle.
run cross_ell    ORBS_BOOT=0 ORBS_GRID=80x45 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend archive; research; attend forge"
# ...and a surface that replaces the pane, which gathers to the middle instead.
# One gather, not three: the union carries the departing session's strip and
# block, and `whole` stands in for them rather than running beside them.
run cross_whole  ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_PASSAGE_AT=0.35 \
  ORBS_DUMP="attend archive; research; wander"
run cross_middle ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_PASSAGE_AT=0.50 \
  ORBS_DUMP="attend archive; research; wander"
# The tower opening out of the boot card, and the only crossing that moves the
# rail. Started by a system on the one frame the sequence hands over, so a dump
# reaches it no other way.
run cross_wake  ORBS_BOOT=0 ORBS_PASSAGE_AT=wake:0.35 ORBS_DUMP="attend laboratory"
run cross_woken ORBS_BOOT=0 ORBS_PASSAGE_AT=wake:0.75 ORBS_DUMP="attend laboratory"
# The off switch, which `tui.sh` and the play suite both set. Byte-identical to
# the same line with no passage variables at all.
run cross_off    ORBS_BOOT=0 ORBS_PASSAGE=0 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend laboratory; attend forge"

echo "captured $(ls -1 "$out"/*.txt | wc -l) screens into $out"
