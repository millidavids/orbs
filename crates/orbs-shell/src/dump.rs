//! The game's own screen, as text, with no window and no GPU.
//!
//! A screenshot needs a composited window, and without one is a black PNG that
//! looks like evidence. So this paints the game's own [`Frame`] — the real
//! [`super::paint`], [`Sim`] and [`ScreenLayout`](orbs_render::ScreenLayout) —
//! and prints it, with no `App`. What it cannot show, the CRT and the caret's
//! blink, carries nothing (rule 2).
//!
//! ```text
//! ORBS_DUMP=1 cargo run -p orbs
//! ORBS_DUMP="attend laboratory; grind sage; meditate 10" cargo run -p orbs
//! ORBS_DUMP=1 ORBS_GRID=80x22 cargo run -p orbs
//! ```

use orbs_render::{Frame, GridSize};
use orbs_sim::Sim;

use super::line::Line;
use super::linear::Linear;
use super::screen::Screen;
use super::transition::PaneTransition;
use crate::stage::Stage;

/// The variable that asks for a dump, and optionally what to type first.
const DUMP: &str = "ORBS_DUMP";

/// The variable that overrides the grid, as `COLSxROWS`.
const GRID: &str = "ORBS_GRID";

/// The variable that picks a boot stage to dump.
const BOOT: &str = "ORBS_BOOT";

/// Where every instrument's animation has reached, in seconds. See [`bench()`].
///
/// A misnomer kept, since the See-it lines name it.
const FIRE_PHASE: &str = "ORBS_FIRE_PHASE";

/// How far through the current **world tick** to draw, `0.0`..`1.0`.
///
/// Without it every bar sits on a tick boundary, the jump the creep removes.
/// See [`bench()`].
const TICK: &str = "ORBS_TICK";

/// How far through the mortar's pour to draw, `0.0`..`1.0`. See [`bench()`].
///
/// The pour is an edge, which a dump never observes.
const LOAD: &str = "ORBS_LOAD";

/// How far through the kindling flare to draw, `0.0`..`1.0`. See [`bench()`].
///
/// The flare lasts under a second in a running game and is mostly colour, so it
/// is the one part of the effect a person cannot reliably catch by playing.
const FLARE: &str = "ORBS_FLARE";

/// Whether screens cross at all — `0` or `off` to stop them.
///
/// Apart from `ORBS_PASSAGE_AT`, since a phase of zero is an ordinary phase.
/// For scripts: `scripts/tui.sh` passes it so no screen is read mid-crossing.
pub const PASSAGE: &str = "ORBS_PASSAGE";

/// How far through a crossing to draw — `0.42`, or `wipe:0.42`.
///
/// The last `;`-separated command is held back: the frame before it is kept,
/// and the one after drawn with the crossing posed over it — two screens the
/// game can reach, [`bench()`]'s standard.
///
/// ```text
/// ORBS_BOOT=0 ORBS_PASSAGE_AT=0.30 \
///   ORBS_DUMP="attend laboratory; attend forge" cargo run -p orbs
/// ```
const PASSAGE_AT: &str = "ORBS_PASSAGE_AT";

/// How far through a number's roll to draw, `0.0`..`1.0`.
///
/// Holds back the last command as `ORBS_PASSAGE_AT` does, and draws what it
/// moved part way. That command must be last in `ORBS_DUMP`, not `ORBS_THEN`.
///
/// ```text
/// ORBS_BOOT=0 ORBS_ROLL_AT=0.4 \
///   ORBS_DUMP="debug_renown 0; debug_renown 20" cargo run -p orbs
/// ```
const ROLL_AT: &str = "ORBS_ROLL_AT";

/// A line left **unsubmitted** in the prompt.
///
/// `ORBS_DUMP` submits every segment, so this is the caret, a partial word and
/// the ghost's only gate.
///
/// ```text
/// ORBS_DUMP="attend laboratory" ORBS_LINE="wield mo" cargo run -p orbs
/// ```
const LINE: &str = "ORBS_LINE";

/// How many records to hold back from the newest end.
///
/// The scroll is a keypress, which a dump never presses.
const SCROLL: &str = "ORBS_SCROLL";

/// What to type into the spell editor, once `scribe` has opened it.
///
/// Newline-separated keystrokes, in order. The editor's own state decides what
/// a segment is — it opens in command state, so the first segment is a word,
/// `edit` drops into the buffer, and `<esc>` comes back out:
///
/// ```text
/// ORBS_DUMP="scribe morning" \
///   ORBS_EDIT="edit\ngrind sage\n<esc>\nquit" cargo run -p orbs
/// ```
///
/// `quit` is how a dump saves, since the game's pause-then-save runs on `Time`.
/// `w` and `wq` also flush.
const EDIT: &str = "ORBS_EDIT";

/// Commands to run **after** `ORBS_EDIT` has finished with the editor.
///
/// A save lands on the next tick, so a `peruse` in `ORBS_DUMP` would run too
/// early:
///
/// ```text
/// ORBS_DUMP="scribe morning" ORBS_EDIT="edit\nbrew clarity\n<esc>\nquit" \
///   ORBS_THEN="peruse morning.spell" cargo run -p orbs
/// ```
const THEN: &str = "ORBS_THEN";

/// Keystrokes for the weave screen a `weave` in `ORBS_DUMP` opened.
///
/// `\n`-separated, each a word (run at its end) or an arrow. The arrows do
/// nothing until `ley` or `mastery` enters a track, as in the game.
///
/// ```text
/// ORBS_DUMP="weave" ORBS_WEAVE="mastery\n<right>\ntake" cargo run -p orbs
/// ```
const WEAVE: &str = "ORBS_WEAVE";

/// Words for the orb's menu a `menu` in `ORBS_DUMP` opened.
///
/// `\n`-separated words, Enter implied — [`WEAVE`]'s shape without arrows;
/// `<esc>` leaves. `quit` prints the menu, since a dump hosts no process to
/// end. `ORBS_THRESHOLD=1` puts the threshold's menu up with no word, the only
/// way a capture reaches the screen a player sees first.
///
/// ```text
/// ORBS_DUMP="menu" ORBS_MENU="zorb" cargo run -p orbs
/// ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_MENU="new" cargo run -p orbs
/// ```
const MENU: &str = "ORBS_MENU";

/// Words for the manual an `ORBS_MENU="manual"` opened.
///
/// `\n`-separated: a chapter name opens it, `<pgdn>` and `<pgup>` page it,
/// `<down>` and `<up>` step a row, `<esc>` steps back out — `ORBS_WEAVE`'s
/// shape with a reader's keys.
///
/// Behind `ORBS_MENU="manual"`, the way a player reaches it.
///
/// ⚠ `<pgdn>` steps a *row* here: a dump measures the pane only when it paints,
/// after the script. Paging by a screenful is the play suite's
/// (`routing::the_manual_pages_and_the_transcript_behind_it_does_not`).
///
/// ```text
/// ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="keys" cargo run -p orbs
/// ```
const MANUAL: &str = "ORBS_MANUAL";

/// Arrow presses for the stacks a `wander` in `ORBS_DUMP` took the keys for.
///
/// `\n`-separated, one of `<up>`, `<right>`, `<down>`, `<left>` per segment.
/// Anything else ends the walk, the way Escape does.
///
/// No tick per token: an arrow goes through `Sim::walk`, which advances no
/// clock.
///
/// ```text
/// ORBS_DUMP="attend archive; divine; wander" ORBS_WALK="<right>\n<down>" cargo run -p orbs
/// ```
const WALK: &str = "ORBS_WALK";

/// Commands are separated by this, so one shell word can drive a session.
const SEPARATOR: char = ';';

/// The grid a dump uses unless asked otherwise: **the one the game draws**.
///
/// 120×45, the game's fixed grid; `ORBS_GRID=80x22` is §4's floor check.
const DEFAULT_GRID: GridSize = orbs_render::GRID;

/// Draw one frame as text if `ORBS_DUMP` asked for it.
///
/// Returns whether it did, so a frontend can exit before building an `App` or
/// entering raw mode. `engine` is the POST card's third line, which only the
/// caller knows. No seed: a dump always draws `crate::seed()`'s world.
#[must_use]
pub fn run(wizard: Option<String>, engine: &str, settings: Vec<crate::settings::Row>) -> bool {
    let Some(request) = requested() else {
        return false;
    };
    run_script(wizard, engine, settings, &request);
    true
}

/// What `ORBS_DUMP` asked for, if it asked for anything.
///
/// Split out for `orbs-tui --dump`, since setting a variable is `unsafe` in
/// edition 2024.
#[must_use]
pub fn requested() -> Option<String> {
    std::env::var(DUMP).ok()
}

/// Draw one frame as text, from a script the caller already has.
///
/// The boundary proof: `orbs-tui --dump X` and `ORBS_DUMP=X orbs` print the
/// same bytes.
pub fn run_script(
    wizard: Option<String>,
    engine: &str,
    settings: Vec<crate::settings::Row>,
    request: &str,
) {
    // The instrument's seed, never a game's — see `crate::seed`.
    let seed = crate::seed();
    // No clock either, unless `ORBS_DATE` names a day: `moon` at the menu
    // would otherwise make a capture depend on when it was taken.
    crate::sky::cover();
    // A dump neither loads nor saves unless `ORBS_SAVE` names a path, or no
    // capture is a baseline. Sealed for the whole process, since the menu's
    // play page reads saves itself. `save::named`, so a blank `ORBS_SAVE=` is
    // no path — it once let a capture overwrite the player's tower.
    let asked = crate::save::named();
    if !asked {
        crate::save::seal();
    }
    let waiting = if asked {
        crate::save::read()
    } else {
        crate::save::Opened::New
    };
    let mut installing = None;
    let mut sim = match waiting {
        crate::save::Opened::Restored(save) => {
            let mut resumed = Sim::restored(&save);
            resumed.say_resumed(crate::save::away_for(&save));
            resumed
        }
        crate::save::Opened::Unreadable => {
            // Baseline: all 138 captures read the authored curve, and a default
            // length would move every one of them the day a tier was retuned.
            let mut fresh =
                crate::environment::fresh(seed, false, orbs_sim::content::Length::Baseline);
            if let Some(name) = wizard {
                fresh.rename(&name);
            }
            fresh.say_save_unreadable();
            fresh
        }
        // `ORBS_INSTALL=1`: the practice tower, with the strip over it.
        crate::save::Opened::New if crate::install::posed() => {
            let (sandbox, install) = crate::install::pose_for_dump(seed, wizard.as_deref());
            installing = install;
            sandbox
        }
        crate::save::Opened::New => {
            // Baseline: all 138 captures read the authored curve, and a default
            // length would move every one of them the day a tier was retuned.
            let mut fresh =
                crate::environment::fresh(seed, false, orbs_sim::content::Length::Baseline);
            // Only a *new* world takes its wizard from the environment: a save
            // carries one, and `session::Wizard` is explicit that a save
            // outranks the machine.
            if let Some(name) = wizard {
                fresh.rename(&name);
            }
            fresh
        }
    };
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6), so a
    // writer sees their edit — and a malformed file says so, since a dump has
    // no `tracing` subscriber.
    for file in crate::prose::load(&mut sim) {
        eprintln!(
            "warn: {} is set but {file} did not load; drawing the built-in text",
            crate::prose::CONTENT_DIR
        );
    }

    let grid = grid();
    // Wide, which every running frontend opens in, not derived from the grid.
    // Only the labels differ while there is one pane (§19).
    let screen = Screen::windowless(grid, None);

    // Settled unless `ORBS_PASSAGE_AT` asks otherwise: a dump is a still, and a
    // still of a screen half gone is a picture of a moment rather than of the
    // screen.
    let mut passing = super::passing::Passing::default();

    // `1` is the idiom for "just boot it"; anything else is a session to type.
    // Every line goes through `submit` and a real `step`, so what prints is the
    // world having actually run rather than a pose struck for the screenshot.
    let posed = requested_crossing();
    let rolled = number(ROLL_AT);
    let mut climb = super::climb::Climb::default();
    if request != "1" {
        if posed.is_some() || rolled.is_some() {
            // The last command held back, so a crossing has a screen to leave
            // and a roll a number that moved.
            let (head, last) = split_last(request);
            drive(&mut sim, &mut installing, head);
            if posed.is_some() {
                leaving(&sim, installing.as_ref(), &screen, grid, &mut passing);
            }
            let mut seen = super::glance::Panel::default();
            if rolled.is_some() {
                seen.refresh(&sim);
                climb.set_enabled(true);
                climb.observe(&seen);
            }
            drive(&mut sim, &mut installing, last);
            // Observed here, before `ORBS_THEN` and its extra tick: whatever
            // those move belongs to the arriving screen and draws settled.
            if rolled.is_some() {
                seen.refresh(&sim);
                climb.observe(&seen);
            }
        } else {
            drive(&mut sim, &mut installing, request);
        }
    }

    // Written after the script has run, because what a dump is *for* is the
    // world the script reached. There is no `quit` involved: a dump has no
    // session to leave, and `run_script` never reads `Quitting`. Never a practice
    // tower: it is kept nowhere, here as in the game.
    if asked && installing.is_none() && crate::save::write(&sim.snapshot()).is_err() {
        // In voice, not on stderr — this runs before the frame is painted, so
        // it lands on the transcript where §3 says output belongs. A dump is
        // also the one place a person is *looking* for what the orb said.
        sim.say_save_failed();
    }

    let mut frame = Frame::new(grid);
    // `ORBS_BOOT=frame` dumps that stage instead of the game. Boot runs once per
    // launch and the window will not composite in a detached shell, so without
    // this the sequence could only be checked by a person sitting in front of it
    // — which is exactly the position `ORBS_DUMP` exists to get out of.
    if let Some((stage, progress)) = requested_stage() {
        super::prompt::paint_booting(&mut frame, stage, progress, engine);
        print(&frame);
        return;
    }
    // The same branch `render::redraw` takes, and for the same reason: below the
    // floor the game draws a "too small" screen rather than a mangled layout, and
    // a tool documented as drawing the same frame the game draws has to draw that
    // one too. `ORBS_GRID=40x10` is how anyone would ever look at it.
    if screen.is_hostable() {
        // Settled, since a dump is a still. One pane at every grid; whether
        // the rail fits is `ScreenLayout::compute`'s.
        let panes = PaneTransition::settled(1);
        let typed = std::env::var(LINE).map_or_else(|_| Line::default(), |text| Line::typed(&text));
        // If a `scribe` in `ORBS_DUMP` asked for the editor, open it and let
        // `ORBS_EDIT` type into it. Without this the surface needs a person at
        // a window.
        let mut editing = opened(&mut sim);
        // ...and the same for a `weave`. Taken before `ORBS_THEN` runs, so a
        // dump can open the screen and then keep issuing commands behind it —
        // which is what the world ticking behind a modal surface looks like.
        let mut weaving = woven(&mut sim);
        // ...and the same for a `wander`, except that this one owns no surface,
        // so what it produces is a flag and some steps already walked.
        let mut walking = walked(&mut sim);
        // ...and the same for a `menu`. Taken here so one in `ORBS_DUMP` and one
        // in `ORBS_THEN` both reach it.
        let mut opened_manual = false;
        let mut menuing = menued(&mut sim, &settings, &mut opened_manual);
        // Commands to run *after* the editing session. A `:w` queues its write
        // for the next tick, so a `peruse` in `ORBS_DUMP` runs before the spell
        // exists and offers the other readables — which looks like a bug. The
        // only way to look at what was just saved.
        if let Ok(after) = std::env::var(THEN) {
            sim.step();
            drive(&mut sim, &mut installing, &after);
            // A `scribe` in `ORBS_THEN` opens the editor too — the only way to
            // show a running spell edited — but untyped: `ORBS_EDIT` belongs
            // to the first `scribe`.
            editing = editing.or_else(|| open(&mut sim));
            weaving = weaving.or_else(|| woven(&mut sim));
            walking |= walked(&mut sim);
            menuing = menuing.or_else(|| menued(&mut sim, &settings, &mut opened_manual));
        }
        // A maze can close under the walker; both frontends then give the
        // keyboard back.
        if walking && sim.stacks().is_none() {
            walking = false;
        }
        // After the menu, because the menu is what opens it. A dump can run the
        // reader for real — it needs the book and nothing else — which is why
        // this is the one menu outcome a capture acts on.
        let mut reading_manual = read_manual(&sim, opened_manual);
        // The world may have moved while the screen was up — `ORBS_THEN` steps.
        // In the game `weaving::refresh` runs every frame for exactly this.
        if let Some(screen) = weaving.as_mut() {
            screen.read_from(&sim);
        }
        // The running-line marker, the reading and the guide, which the game
        // pushes in on `Time`: from the same accessors here, or `interpret`
        // draws empty and the guide is a blank box.
        if let Some(editor) = editing.as_mut() {
            editor.set_running_line(sim.running_line(editor.name()));
            // Through the scrivener, so a capture can show a read line. The
            // reading is what `interpret` draws, and a dump that never consulted
            // one would be blind to the whole surface — CLAUDE.md's *"the
            // blindness looks exactly like stability"*.
            editor.set_reading(match super::environment::scrivener() {
                Some(reader) => sim.read_spell_with(
                    editor.name(),
                    editor.domain(),
                    editor.lines(),
                    reader.as_ref(),
                ),
                None => sim.read_spell(editor.domain(), editor.lines()),
            });
            editor.refresh(&sim);
        }
        // A dump builds no `App`, so the two cached resources are computed here
        // from the same functions the systems call: one silently omitted is a
        // picture that proves the wrong thing. Taken before the borrow below,
        // because `unfurling` takes rather than reads and `View` holds `&sim`.
        let scroll = scrolled(&mut sim);
        // Through `Panel::refresh`, not a second literal: the literal this was
        // listed nine fields by hand, so a tenth could be added to the resource
        // and forgotten here.
        let mut panel = super::glance::Panel::default();
        panel.refresh(&sim);
        if let Some(fraction) = rolled {
            climb.pose(fraction);
        }
        // Posed here rather than where the script ran: whether the transcript
        // is spared turns on which surface ended up open — the same question
        // `Showing::replaces_the_pane` asks in the game — and the three flags
        // that answer it are only resolved by now.
        if let Some(Posed {
            passage,
            progress,
            waking,
        }) = posed
        {
            // A surface open means the last command took the pane, which is the
            // same question `Change` asks in the game and the same answer it
            // gives. Which *regions* move is `prompt::paint`'s, and it works it
            // out from the screen it just drew rather than from this.
            let chosen = if editing.is_some() || weaving.is_some() || walking {
                orbs_render::Passage::Gather
            } else {
                orbs_render::Passage::Wipe
            };
            if waking {
                passing.pose_wake(progress);
            } else {
                passing.pose(progress, passage.unwrap_or(chosen));
            }
        }
        super::prompt::paint(
            &mut frame,
            &mut Linear::default(),
            &mut passing,
            super::prompt::View {
                sim: &sim,
                line: &typed,
                screen: &screen,
                panes: &panes,
                // A dump is a still. `Reveal::default()` has nothing in flight,
                // so the output it prints is the output that finished arriving.
                reveal: &super::reveal::Reveal::default(),
                // Tab's candidate list is a keystroke's worth of state, and a
                // dump presses no keys. The *ghost* still shows, because it is a
                // function of `ORBS_LINE` rather than of anything that happened.
                offered: &super::offering::Offered::default(),
                scroll: &scroll,
                ghost: &typed.ghost(sim.scene(), !sim.choices().is_empty()),
                panel: &panel,
                // A dump advances no `Time`, so every instrument would draw at
                // phase zero for ever and the See-it gate would be "it
                // compiles". `ORBS_FIRE_PHASE` steps them; see CLAUDE.md.
                bench: &bench(),
                // Settled unless `ORBS_ROLL_AT` posed it.
                climb: &climb,
                editing: editing.as_mut(),
                weaving: weaving.as_ref(),
                walking,
                menuing: menuing.as_ref(),
                reading_manual: reading_manual.as_mut(),
                installing: installing.as_ref(),
            },
        );
    } else {
        super::prompt::paint_too_small(&mut frame, &sim);
    }

    print(&frame);
}

/// A wash's colour, as a writer would name it.
///
/// Two names joined for the flask's mixing band, which is two materials.
fn label(wash: orbs_render::Wash) -> String {
    match wash.with {
        None => wash.tint.name().to_owned(),
        Some(second) => format!("{}+{}", wash.tint.name(), second.name()),
    }
}

/// The frame, then what it says, then what it is tinted.
fn print(frame: &Frame) {
    println!("{}", frame.to_text());
    println!("-- linearised (DESIGN.md §14) --");
    for utterance in frame.speech().utterances() {
        println!(
            "  {:<10} {}",
            format!("{:?}", utterance.kind),
            utterance.text
        );
    }

    // A tint is colour the glyphs above cannot show. Printed only when there
    // are some.
    if !frame.tints().is_empty() {
        println!("-- tinted regions (DESIGN.md §19) --");
        for (area, wash) in frame.tints() {
            println!(
                "  {:<8} {}×{} at {},{}",
                label(*wash),
                area.cols,
                area.rows,
                area.col,
                area.row,
            );
        }
    }

    // And the spell's parts of speech, for the tints' reason — the only text
    // gate the Bevy build has. In reading order, so a misclassified word
    // shows beside its neighbours.
    if !frame.syntax().is_empty() {
        println!("-- lit runs (DESIGN.md §19) --");
        for (area, kind) in frame.syntax() {
            let word: String = (area.col..area.right())
                .filter_map(|col| frame.cell(orbs_render::Pos::new(col, area.row)))
                .map(|cell| cell.glyph)
                .collect();
            println!("  {:<8} {word:?} at {},{}", part(*kind), area.col, area.row);
        }
    }
}

/// A part of speech, as the dump names it.
///
/// Spelled out rather than `{:?}`'d, so the column reads as prose and a rename
/// of the enum does not silently rewrite every captured baseline.
const fn part(kind: orbs_render::Lexeme) -> &'static str {
    match kind {
        orbs_render::Lexeme::Control => "control",
        orbs_render::Lexeme::Verb => "verb",
        orbs_render::Lexeme::Name => "name",
        orbs_render::Lexeme::Number => "number",
        orbs_render::Lexeme::Comment => "comment",
        orbs_render::Lexeme::Call => "call",
        orbs_render::Lexeme::Filler => "filler",
        orbs_render::Lexeme::Grammar => "grammar",
        orbs_render::Lexeme::State => "state",
        orbs_render::Lexeme::None => "none",
    }
}

/// The boot stage `ORBS_BOOT` asked for, and how far through it.
///
/// `ORBS_BOOT=frame` is halfway through that stage, the only interesting point
/// for something that animates. `ORBS_BOOT=0` skips it entirely and belongs to
/// the running game.
///
/// A trailing `:<fraction>` names how far, so `ORBS_BOOT=post:1` is the
/// finished card with its version lines.
fn requested_stage() -> Option<(Stage, f32)> {
    let request = std::env::var(BOOT).ok()?;
    let (name, asked) = match request.split_once(':') {
        Some((name, fraction)) => (name, fraction.trim().parse::<f32>().ok()),
        None => (request.as_str(), None),
    };
    // `frame` outlived the stage it named. The border and the card are one
    // stage now, so it selects the moment the box is still closing and the
    // first letter landing — what anyone typing `frame` wanted, and never a
    // thing the old stage could show.
    let (stage, progress) = match name {
        "dark" => (Stage::Dark, 0.5),
        "frame" => (Stage::Post, Stage::FRAME_SHARE / 2.0),
        "post" => (Stage::Post, 0.5),
        // The card leaving, an animation and so needing a fraction more than
        // the others: bare `close` is the middle of the collapse, `close:1` the
        // single cell it ends on.
        "close" => (Stage::Close, 0.5),
        _ => return None,
    };
    Some((stage, asked.unwrap_or(progress).clamp(0.0, 1.0)))
}

/// A script split into everything but its last command, and that command.
///
/// The last **non-empty** segment, so a trailing `;` does not hand back an empty
/// command and a crossing out of the finished screen into itself.
fn split_last(script: &str) -> (&str, &str) {
    let trimmed = script.trim_end().trim_end_matches(SEPARATOR).trim_end();
    match trimmed.rfind(SEPARATOR) {
        Some(at) => (&trimmed[..at], &trimmed[at + 1..]),
        None => ("", trimmed),
    }
}

/// Paint the screen a posed crossing departs from, and keep it.
///
/// The plain session: the surfaces belong to the arriving screen.
fn leaving(
    sim: &Sim,
    installing: Option<&crate::install::Install>,
    screen: &Screen,
    grid: orbs_render::GridSize,
    passing: &mut super::passing::Passing,
) {
    if !screen.is_hostable() {
        return;
    }
    let mut frame = Frame::new(grid);
    let mut panel = super::glance::Panel::default();
    panel.refresh(sim);
    // Settled, and it must be: this is the frame a crossing departs from, so a
    // crossing drawn over it would be a picture of two. It also records the
    // departing screen's *regions*, which is why the posed `Passing` is handed
    // in rather than a throwaway — `paint` writes them back through it.
    super::prompt::paint(
        &mut frame,
        &mut Linear::default(),
        passing,
        super::prompt::View {
            sim,
            line: &Line::default(),
            screen,
            panes: &PaneTransition::settled(1),
            reveal: &super::reveal::Reveal::default(),
            offered: &super::offering::Offered::default(),
            ghost: "",
            panel: &panel,
            scroll: &super::scrollback::Scroll::default(),
            bench: &super::bench::Bench::default(),
            climb: &super::climb::Climb::default(),
            editing: None,
            weaving: None,
            walking: false,
            menuing: None,
            reading_manual: None,
            installing,
        },
    );
    passing.pose_kept(&frame);
}

/// The shape and fraction `ORBS_PASSAGE_AT` asks for, if it asks.
///
/// `requested_stage`'s shape. A bare fraction poses the shape the game would
/// choose; a name overrides it. Anything malformed poses nothing.
fn requested_crossing() -> Option<Posed> {
    use orbs_render::Passage;

    let request = std::env::var(PASSAGE_AT).ok()?;
    let (name, fraction) = match request.split_once(':') {
        Some((name, fraction)) => (Some(name.trim()), fraction),
        None => (None, request.as_str()),
    };
    let (passage, waking) = match name {
        None => (None, false),
        Some("wipe") => (Some(Passage::Wipe), false),
        Some("furl") => (Some(Passage::Furl), false),
        Some("gather") => (Some(Passage::Gather), false),
        // The one that arrives out of the boot card, and the only crossing that
        // moves the tower rail. A dump reaches it no other way: a system starts
        // it, on the one frame the sequence hands over.
        Some("wake") => (Some(Passage::Wipe), true),
        Some(_) => return None,
    };
    let parsed = fraction.trim().parse::<f32>().ok()?;
    parsed.is_finite().then(|| Posed {
        passage,
        progress: parsed.clamp(0.0, 1.0),
        waking,
    })
}

/// What `ORBS_PASSAGE_AT` asked for.
///
/// A struct rather than a tuple: the three are a shape, a fraction and a flag,
/// and `View`'s own doc names what a positional list of those invites.
struct Posed {
    /// The shape, or `None` to take whichever the change itself would choose.
    passage: Option<orbs_render::Passage>,
    /// How far through, `0.0`..`1.0`.
    progress: f32,
    /// Whether this is the crossing that arrives out of boot.
    waking: bool,
}

/// Whether crossings run at all. See [`PASSAGE`].
pub(crate) fn passage_permitted() -> bool {
    !matches!(
        std::env::var(PASSAGE).as_deref().map(str::trim),
        Ok("0" | "off" | "false")
    )
}

/// A finite `f32` from the environment, if the variable holds one.
///
/// One parse for four switches; `NaN` survives `clamp`, so `is_finite` matters.
fn number(name: &str) -> Option<f32> {
    let value = std::env::var(name).ok()?;
    let parsed = value.trim().parse::<f32>().ok()?;
    parsed.is_finite().then_some(parsed)
}

/// The athanor's fire, at whatever phase `ORBS_FIRE_PHASE` asks for.
///
/// A dump advances no `Time`, so stepping this is how the plume moves:
///
/// ```text
/// ORBS_DUMP="attend laboratory; kindle charcoal; meditate 300" \
///   ORBS_FIRE_PHASE=0.4 cargo run -p orbs
/// ```
///
/// A malformed value burns at zero, as a malformed grid falls back.
/// `ORBS_FIRE=0` turns the effect off entirely — separate switches, because a
/// phase of zero is an ordinary phase.
fn bench() -> super::bench::Bench {
    let mut bench = super::bench::Bench::default();
    if let Some(phase) = number(FIRE_PHASE) {
        // A negative phase is refused by `tick` itself rather than here — see
        // `Bench::tick`, where running the decays backwards used to manufacture
        // an ignition and a pour out of nothing.
        bench.tick(phase);
    }
    // After the phase, because `tick` decays the flare: setting it first would
    // have the phase burn it off, and `ORBS_FLARE=1` would silently do nothing
    // at any phase past a second.
    if let Some(fraction) = number(FLARE) {
        bench.set_flare(fraction);
    }
    if let Some(fraction) = number(TICK) {
        bench.set_advance(fraction.clamp(0.0, 1.0));
    }
    if let Some(fraction) = number(LOAD) {
        bench.set_load(fraction);
    }
    bench
}

/// How far back `ORBS_SCROLL` asks the transcript to be.
///
/// A malformed value scrolls nowhere, for the same reason a malformed grid falls
/// back: the useful answer to a typo is the default screen, not a stack trace.
fn scrolled(sim: &mut orbs_sim::Sim) -> super::scrollback::Scroll {
    let mut scroll = super::scrollback::Scroll::default();
    if let Ok(back) = std::env::var(SCROLL)
        && let Ok(back) = back.trim().parse::<u16>()
    {
        // Clamped where the painter clamps: one record kept on screen.
        let total = sim.scrollback().records().drawn_len();
        scroll.page(usize::from(back), true, total);
    }
    // An `unfurl` in the script hands the transcript the keyboard, as
    // `plugin::start_reading` does in the game; a dump builds no `App`, so that
    // system has nobody to run it. The two compose: `ORBS_SCROLL` says how far
    // back to look, `unfurl` says the keys are live.
    if sim.unfurling() {
        scroll.read();
    }
    scroll
}

/// Submit every `;`-separated command in `script`, stepping between them.
///
/// One loop for before and after an editing session. While `installing`, a
/// line goes through the installation, as in both frontends.
fn drive(sim: &mut Sim, installing: &mut Option<crate::install::Install>, script: &str) {
    // Read once, for the whole script. `ORBS_AUGURY` is off unless asked for,
    // so an older dump drives as it always did, and one with `stub` set reaches
    // a divined line — the only way `dumps.sh` sees that surface at all.
    let augury = super::environment::augury();
    for line in script
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if installing.is_some() {
            crate::install::drive_dump(sim, installing, line, augury.as_deref());
            continue;
        }
        sim.submit_with(line, augury.as_deref());
        sim.step();
    }
}

/// The editor a pending `scribe` asked for, with nothing typed into it.
///
/// `editing::open_requested`'s two lines; apart from [`opened`], since a later
/// `scribe` opens a buffer rather than replaying `ORBS_EDIT`.
fn open(sim: &mut orbs_sim::Sim) -> Option<super::Editor> {
    let request = sim.opening()?;
    Some(super::Editor::open(
        &request.name,
        &request.domain,
        &request.lines,
    ))
}

/// The editor, if a `scribe` in this dump opened one, with `ORBS_EDIT` typed in.
///
/// The keystrokes replay through the [`Editor`](super::Editor) methods the key
/// handler calls, every character through `insert`, so dump and game agree.
/// The editor's state says what a segment is. A save writes for real.
fn opened(sim: &mut orbs_sim::Sim) -> Option<super::Editor> {
    let mut editor = open(sim)?;

    let Ok(script) = std::env::var(EDIT) else {
        return Some(editor);
    };
    // `\n` as two characters, because a shell word carries it that way.
    let mut wrote_a_line = false;
    for segment in script.replace("\\n", "\n").split('\n') {
        // The one token a keyboard has and a shell word does not.
        if segment.trim() == "<esc>" {
            editor.escape();
            continue;
        }

        // In the buffer, a break goes **between** lines rather than after each
        // one: a trailing newline would put an empty line at the end of every
        // spell the dump writes, and the log would report one line more than was
        // typed.
        if editor.mode() == super::EditorMode::Editing && wrote_a_line {
            editor.enter();
        }
        for character in segment.chars() {
            editor.type_text(&character.to_string());
        }

        if editor.mode() == super::EditorMode::Editing {
            wrote_a_line = true;
            continue;
        }
        // Command state: `Enter` runs the word.
        match editor.enter() {
            Some(super::EditorOutcome::Save) => {
                wrote(sim, &editor);
                editor.saved();
            }
            Some(super::EditorOutcome::SaveAndClose) => {
                wrote(sim, &editor);
                sim.step();
                return None;
            }
            None => {}
        }
    }
    Some(editor)
}

/// The arrow keys a `wander` in the dump asked for, with `ORBS_WALK` played
/// into them.
///
/// Through `Sim::walk`, a key press's door, for [`opened`]'s reason. Returns
/// whether the arrows still have the maze, for the status row.
fn walked(sim: &mut orbs_sim::Sim) -> bool {
    if !sim.wandering() {
        return false;
    }
    let Ok(script) = std::env::var(WALK) else {
        return true;
    };
    for key in keystrokes(&script, ARROWS) {
        // Anything but an arrow ends the walk, which is what Escape does.
        let Some(way) = super::apply_to_maze(&key) else {
            return false;
        };
        // `walk`, not `submit` and `step`: a tick per arrow meant eight presses
        // advanced the world eight seconds, so a brew could finish and a fire
        // burn down inside what is meant to be a still.
        if !sim.walk(way) {
            return false;
        }
    }
    true
}

/// Save the buffer, through whatever scrivener the environment named.
///
/// One place for `w` and `wq`, so the spell never depends on how it closed.
fn wrote(sim: &mut orbs_sim::Sim, editor: &super::Editor) {
    match super::environment::scrivener() {
        Some(reader) => sim.write_spell_reading(editor.name(), editor.lines(), reader.as_ref()),
        None => sim.write_spell(editor.name(), editor.lines()),
    }
}

/// The orb's menu, if a `quit` in the script opened it, with [`MENU`] typed in.
///
/// `sim.menuing()` takes the handshake, so it fires once per `menu`, as in the
/// game.
fn menued(
    sim: &mut orbs_sim::Sim,
    settings: &[crate::settings::Row],
    opened: &mut bool,
) -> Option<super::Menu> {
    // `ORBS_THRESHOLD=1` opens it with no word, in the threshold's stance — no
    // `resume`, and `<esc>` does not close it.
    let stance = crate::Threshold::chosen(crate::Threshold::Playing);
    let stance = if stance.is_waiting() {
        crate::Stance::Threshold
    } else {
        crate::Stance::InTower
    };
    if stance == crate::Stance::InTower && !sim.menuing() {
        return None;
    }
    // The caller's settings, as the engine line is: each binary hands over its
    // first launch's, which keeps the capture reproducible.
    let mut menu = super::Menu::open_at(stance, super::Driver::default(), settings.to_vec());

    let Ok(script) = std::env::var(MENU) else {
        return Some(menu);
    };
    for key in keystrokes(&script, MENU_KEYS) {
        match super::apply_to_menu(&key, &mut menu) {
            // `resume` really closes it, so `ORBS_MENU="resume"` draws the
            // tower — the half of the See-it line that proves the menu is a
            // place you can leave.
            Some(super::MenuOutcome::Close) => return None,
            // The manual really opens, unlike the outcomes below: it needs only
            // the book. `opened` is where the reader ends up; `menued` returns
            // the menu it is over.
            Some(super::MenuOutcome::OpenManual) => *opened = true,
            // Nothing to end or swap — see [`MENU`] — so the menu stays on
            // screen. `Drive` too: the menu already wrote the setting.
            Some(
                super::MenuOutcome::Set { .. }
                | super::MenuOutcome::PutDown
                | super::MenuOutcome::Raise(_)
                | super::MenuOutcome::Drive(_),
            )
            | None => {}
        }
    }
    Some(menu)
}

/// The manual a `manual` in `ORBS_MENU` opened, driven by `ORBS_MANUAL`.
///
/// The reader really runs, since it needs only the book; `<esc>` from the
/// contents draws the menu behind.
fn read_manual(sim: &orbs_sim::Sim, opened: bool) -> Option<super::ManualReader> {
    if !opened {
        return None;
    }
    let mut reader = super::ManualReader::of(super::manual_book(sim));
    let Ok(script) = std::env::var(MANUAL) else {
        return Some(reader);
    };
    for key in keystrokes(&script, MANUAL_KEYS) {
        if super::apply_to_manual(&key, &mut reader).is_some() {
            // Closed. The menu it sat over is what a capture then shows.
            return None;
        }
    }
    Some(reader)
}

/// The weave screen a `weave` in the dump asked for, with `ORBS_WEAVE` played
/// into it.
///
/// Through the key handler's `Tapestry` methods, for [`opened`]'s reason. The
/// reading is pushed first and last, as `weaving::refresh` does each frame.
fn woven(sim: &mut orbs_sim::Sim) -> Option<super::Tapestry> {
    if !sim.weaving() {
        return None;
    }
    let mut screen = super::Tapestry::default();
    screen.read_from(sim);

    let Ok(script) = std::env::var(WEAVE) else {
        return Some(screen);
    };
    for key in keystrokes(&script, WEAVE_KEYS) {
        match super::apply_to_weave(&key, &mut screen) {
            Some(super::WeaveOutcome::Close) => return None,
            // The dump takes it too, or `ORBS_WEAVE="mastery\ntake"` would draw
            // a screen where nothing happened — the See-it line for the whole
            // progression tree. It queues, and `ORBS_THEN` is where the granting
            // tick comes from.
            Some(super::WeaveOutcome::Take(id)) => sim.take(&id),
            None => {}
        }
    }
    screen.read_from(sim);
    Some(screen)
}

/// The arrow tokens, which `ORBS_WALK` reads and nothing else.
const ARROWS: &[&str] = &["<up>", "<down>", "<left>", "<right>"];

/// The tokens `ORBS_MENU` reads as keys.
const MENU_KEYS: &[&str] = &["<esc>"];

/// The tokens `ORBS_MANUAL` reads as keys.
const MANUAL_KEYS: &[&str] = &["<esc>", "<up>", "<down>", "<pgup>", "<pgdn>"];

/// The tokens `ORBS_WEAVE` reads as keys.
const WEAVE_KEYS: &[&str] = &["<esc>", "<up>", "<down>", "<left>", "<right>"];

/// A dump's key script as keystrokes, for the same `apply_to_*` table the
/// frontends drive: the tokens in `known` as the keys they name, and any other
/// segment typed and then entered — there is no buffer outside the editor, so a
/// segment can only mean *a word, now run it*.
///
/// `known` is per surface, since a token one surface never read is a word to
/// another: `ORBS_MENU='<down>'` draws the menu's complaint about it.
///
/// `\n` as two characters, because a shell word carries it that way.
fn keystrokes(script: &str, known: &[&str]) -> Vec<super::Key> {
    let mut keys = Vec::new();
    for segment in script.replace("\\n", "\n").split('\n') {
        let segment = segment.trim();
        let named = match segment {
            "" => continue,
            token if !known.contains(&token) => None,
            "<esc>" => Some(super::Key::Escape),
            "<up>" => Some(super::Key::Up),
            "<down>" => Some(super::Key::Down),
            "<left>" => Some(super::Key::Left),
            "<right>" => Some(super::Key::Right),
            "<pgup>" => Some(super::Key::PageUp),
            "<pgdn>" => Some(super::Key::PageDown),
            _ => None,
        };
        let Some(named) = named else {
            keys.extend(
                segment
                    .chars()
                    .map(|glyph| super::Key::Text(glyph.to_string())),
            );
            keys.push(super::Key::Enter);
            continue;
        };
        keys.push(named);
    }
    keys
}

/// The grid to draw into, from `ORBS_GRID` or §4's floor.
///
/// A malformed value falls back: the useful answer to a typo is the default
/// screen.
fn grid() -> GridSize {
    let Ok(request) = std::env::var(GRID) else {
        return DEFAULT_GRID;
    };
    let Some((cols, rows)) = request.split_once(['x', 'X']) else {
        return DEFAULT_GRID;
    };
    match (cols.trim().parse(), rows.trim().parse()) {
        (Ok(cols), Ok(rows)) => GridSize { cols, rows },
        _ => DEFAULT_GRID,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Key;

    #[test]
    fn a_token_is_a_key_only_on_a_surface_that_reads_it() {
        let typed = vec![
            Key::Text("<".into()),
            Key::Text("d".into()),
            Key::Text("o".into()),
            Key::Text("w".into()),
            Key::Text("n".into()),
            Key::Text(">".into()),
            Key::Enter,
        ];
        assert_eq!(keystrokes("<down>", MENU_KEYS), typed);
        assert_eq!(keystrokes("<down>", MANUAL_KEYS), vec![Key::Down]);
        assert_eq!(keystrokes("<left>", MANUAL_KEYS).len(), 7);
        assert_eq!(keystrokes("<pgdn>", WEAVE_KEYS).len(), 7);
        assert_eq!(
            keystrokes("<esc>\\nback", MENU_KEYS),
            vec![
                Key::Escape,
                Key::Text("b".into()),
                Key::Text("a".into()),
                Key::Text("c".into()),
                Key::Text("k".into()),
                Key::Enter,
            ],
        );
    }
}
