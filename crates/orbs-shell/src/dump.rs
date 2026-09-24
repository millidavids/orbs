//! The game's own screen, as text, with no window and no GPU.
//!
//! `ORBS_CAPTURE=1` was how work got *looked at*, but it needs a composited
//! window: from a detached shell, or with the display asleep, the screenshot is
//! a valid PNG of a black rectangle — worse than no picture, because it looks
//! like evidence.
//!
//! So this draws the same frame the game draws — [`super::paint`], the real
//! [`Sim`], the real [`ScreenLayout`](orbs_render::ScreenLayout) — into a
//! [`Frame`] nobody rasterises, and prints it. No `App`, no `DefaultPlugins`,
//! no adapter. It cannot show what a frontend owns and the Frame does not:
//! phosphor, the CRT curve, the blinking caret. Rule 2 is the promise that
//! nothing *informational* is among them.
//!
//! ```text
//! ORBS_DUMP=1 cargo run -p orbs
//! ORBS_DUMP="attend laboratory; decoct clarity; meditate 25" cargo run -p orbs
//! ORBS_DUMP=1 ORBS_GRID=120x33 cargo run -p orbs
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
/// A misnomer kept: it drives the mortar's stroke and everything after the fire
/// too, but renaming a variable written down in CLAUDE.md and §15's See-it
/// lines costs more.
const FIRE_PHASE: &str = "ORBS_FIRE_PHASE";

/// How far through the current **world tick** to draw, `0.0`..`1.0`.
///
/// A dump builds no `App`, so with no `Time<Fixed>` every bar would sit on a
/// tick boundary — the one jump the creep exists to remove, and so the one
/// thing about it a dump could not show. See [`bench()`].
const TICK: &str = "ORBS_TICK";

/// How far through the mortar's pour to draw, `0.0`..`1.0`. See [`bench()`].
///
/// The load is an *edge* — the frame a reagent enters an empty bowl — and a dump
/// runs no systems, so it never observes one. Without this the pour is the one
/// animation with no See-it line at all.
const LOAD: &str = "ORBS_LOAD";

/// How far through the kindling flare to draw, `0.0`..`1.0`. See [`bench()`].
///
/// The flare lasts under a second in a running game and is mostly colour, so it
/// is the one part of the effect a person cannot reliably catch by playing.
const FLARE: &str = "ORBS_FLARE";

/// Whether screens cross at all — `0` or `off` to stop them.
///
/// A separate switch from `ORBS_PASSAGE_AT`, on the fire's precedent: a phase
/// of zero is an ordinary phase, and doubly so here, where a crossing at zero
/// is an endpoint that draws the screen exactly.
///
/// For scripted runs rather than players: `scripts/tui.sh` passes it so the
/// play suite cannot read a screen back mid-crossing. A player reaches `F3`.
pub const PASSAGE: &str = "ORBS_PASSAGE";

/// How far through a crossing to draw — `0.42`, or `wipe:0.42`.
///
/// A dump paints one frame, so on its own there is nothing to cross from. When
/// this is set the final `;`-separated command is held back: the frame is
/// painted and kept, the command runs, and it is painted again with the
/// crossing posed over it — so the picture is a crossing between two screens
/// the game can reach, which is [`bench()`]'s standard.
///
/// ```text
/// ORBS_BOOT=0 ORBS_PASSAGE_AT=0.30 \
///   ORBS_DUMP="attend laboratory; attend forge" cargo run -p orbs
/// ```
const PASSAGE_AT: &str = "ORBS_PASSAGE_AT";

/// A line left **unsubmitted** in the prompt.
///
/// `ORBS_DUMP` submits every `;`-separated segment, so the buffer is empty by
/// the time the frame is painted — leaving the caret, a partly-typed word and
/// the suggestion ghost ungated without this.
///
/// ```text
/// ORBS_DUMP="attend laboratory" ORBS_LINE="wield mo" cargo run -p orbs
/// ```
const LINE: &str = "ORBS_LINE";

/// How many records to hold back from the newest end.
///
/// The transcript's scroll is a keypress and a dump presses no keys, so without
/// this §15's *reach it from the running game* needs a person at a window. Same
/// reason `ORBS_LINE` exists.
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
/// `quit` is how a dump saves: the running game writes the buffer out a beat
/// after typing stops, and that pause is measured off `Time`, which a dump
/// never advances. `w` and `wq` also flush, without closing and with.
///
/// Without this the editor needs a person at a window. Same reason `ORBS_LINE`
/// and `ORBS_SCROLL` exist.
const EDIT: &str = "ORBS_EDIT";

/// Commands to run **after** `ORBS_EDIT` has finished with the editor.
///
/// A save queues its write for the next tick, like every other effect (see
/// `session`'s two clocks), so a `peruse` in `ORBS_DUMP` runs before the spell
/// is there. This is what makes the payoff visible:
///
/// ```text
/// ORBS_DUMP="scribe morning" ORBS_EDIT="edit\nbrew clarity\n<esc>\nquit" \
///   ORBS_THEN="peruse morning.spell" cargo run -p orbs
/// ```
const THEN: &str = "ORBS_THEN";

/// Keystrokes for the weave screen a `weave` in `ORBS_DUMP` opened.
///
/// `\n`-separated, every segment a whole thing — a word, or an arrow token.
/// There is no buffer, unlike `ORBS_EDIT`: a word runs when its segment ends,
/// an arrow moves the cursor and runs nothing.
///
/// The arrows do nothing until a word has gone into a track, as in the game —
/// `ley` or `mastery` hands them over, the way `edit` drops into the buffer.
///
/// ```text
/// ORBS_DUMP="weave" ORBS_WEAVE="mastery\n<right>\ntake" cargo run -p orbs
/// ```
const WEAVE: &str = "ORBS_WEAVE";

/// Words for the orb's menu a `menu` in `ORBS_DUMP` opened.
///
/// `\n`-separated, one word per segment, Enter implied at the end of each —
/// [`WEAVE`]'s shape minus the arrows, since the menu has nothing to walk.
/// `<esc>` leaves it, as Escape does.
///
/// Without this the instrument is blind to the one surface `quit` reaches, and
/// CLAUDE.md names that blindness: *"the blindness looks exactly like
/// stability"* — Phase 8 shipped the bailey that way.
///
/// `quit` from the menu prints the menu: a dump hosts no process, so there is
/// nothing for `MenuOutcome::PutDown` to end, and what it means here is *the
/// screen the player was looking at when they left*. `run_script` never read
/// `Quitting` either.
///
/// The threshold's menu needs no word, because nothing types one there:
/// `ORBS_THRESHOLD=1` puts it up and this drives it, the only way a capture
/// reaches the screen a player sees first.
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
/// It needs `ORBS_MENU="manual"` in front of it, because that is how a player
/// reaches the manual and opening it from nowhere would be an instrument
/// reading a screen the game cannot produce.
///
/// ⚠ `<pgdn>` steps a *row* here, not a screen: a page is the pane's height
/// minus one, and a dump takes its one measurement when it paints, after this
/// script has run. The game measures the reader every frame.
///
/// Stated rather than worked around: the alternative is a dump guessing a pane
/// height, and the picture — a chapter wrapping, saying there is more, stepping
/// back on Escape — is what a capture is for. Paging by a real screenful is the
/// play suite's:
/// `routing::the_manual_pages_and_the_transcript_behind_it_does_not`.
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
/// No tick per token, because there is none in the game: an arrow goes through
/// `Sim::walk`, which moves the reading and advances no clock. Stepping between
/// presses would show a world eight seconds older than a player's.
///
/// ```text
/// ORBS_DUMP="attend archive; divine; wander" ORBS_WALK="<right>\n<down>" cargo run -p orbs
/// ```
const WALK: &str = "ORBS_WALK";

/// Commands are separated by this, so one shell word can drive a session.
const SEPARATOR: char = ';';

/// The grid a dump uses unless asked otherwise: **the one the game draws**.
///
/// This was §4's 80×22 floor, so a layout bug showed where things are tightest.
/// That held while the grid followed the window; the game is 120×45 always now,
/// and a dump showing anything else reads a screen nobody has.
/// `ORBS_GRID=80x22` is the floor check, which CLAUDE.md's See-it lines use for
/// width.
const DEFAULT_GRID: GridSize = orbs_render::GRID;

/// Draw one frame as text if `ORBS_DUMP` asked for it.
///
/// Returns whether it did, so a frontend can exit instead of opening a window.
/// Called before the `App` is built, or before raw mode is entered: the point
/// is to need none of it.
///
/// `engine` is the POST card's third line — the one fact only the caller knows.
/// See the POST card's own module for why it cannot be a constant.
///
/// No seed parameter: a dump is an instrument and always draws
/// `crate::seed()`'s world, so a frontend holding a new game's seed cannot hand
/// it one by mistake.
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
/// Split out so a frontend can offer the same thing under its own flag —
/// `orbs-tui --dump "attend laboratory"` — without setting an environment
/// variable, which edition 2024 makes `unsafe` and the workspace denies.
#[must_use]
pub fn requested() -> Option<String> {
    std::env::var(DUMP).ok()
}

/// Draw one frame as text, from a script the caller already has.
///
/// The boundary proof: both binaries reach it, through the same painters, from
/// the same `Sim` — so `orbs-tui --dump X` and `ORBS_DUMP=X orbs` print the
/// same bytes or the shell has grown a frontend-shaped hole in it.
pub fn run_script(
    wizard: Option<String>,
    engine: &str,
    settings: Vec<crate::settings::Row>,
    request: &str,
) {
    // The instrument's seed, never a game's — see `crate::seed`.
    let seed = crate::seed();
    // A dump neither loads nor saves unless `ORBS_SAVE` names a path. Loading
    // by default would make every See-it line depend on whether anyone had
    // played there, and `dumps.sh` would stop being a baseline the first time
    // one of its 56 screens wrote a save the next 55 then read.
    //
    // Sealed for the whole process, not just this line: the menu's play page
    // calls `save::path()`, `save::saves()` and `save::abandon()` itself, so
    // without the seal a capture listed the player's real towers and
    // `ORBS_MENU=$'play\nabandon 1\nabandon 1'` renamed one. Both reproduced.
    //
    // `save::named`, not `var_os(..).is_some()`: an exported-but-empty
    // `ORBS_SAVE=` is not a path a person asked for, and reading it as one
    // skipped the seal while `save::chosen` resolved it to the real app-data
    // directory — so the capture overwrote the player's tower.
    let asked = crate::save::named();
    if !asked {
        crate::save::seal();
    }
    let waiting = if asked {
        crate::save::read()
    } else {
        crate::save::Opened::New
    };
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
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6). A dump
    // has no watcher but must still read what is on disk, or the first tool
    // CLAUDE.md names is the one that cannot show a writer their own edit. It
    // installs no `tracing` subscriber either, so without these lines malformed
    // TOML looks exactly like the built-in fallback.
    for file in crate::prose::load(&mut sim) {
        eprintln!(
            "warn: {} is set but {file} did not load; drawing the built-in text",
            crate::prose::CONTENT_DIR
        );
    }

    let grid = grid();
    // Wide, which is what every running frontend opens in. Deriving the mode
    // from the grid instead drew the *opposite* focus mode to the game: a dump
    // printed `focus deep` and `F4 wide` where the terminal printed
    // `focus wide` and `F4 deep`.
    //
    // Only the two labels today, since one pane makes both tilings identical
    // and §19 records `F4` as inert until Phase 11a returns the second pane.
    // When it does, every dump would show a layout the game never draws — and
    // CLAUDE.md quotes this output: *"a See-it line that describes a different
    // screen is worse than none."*
    let screen = Screen::windowless(grid, None);

    // Settled unless `ORBS_PASSAGE_AT` asks otherwise: a dump is a still, and a
    // still of a screen half gone is a picture of a moment rather than of the
    // screen.
    let mut passing = super::passing::Passing::default();

    // `1` is the idiom for "just boot it"; anything else is a session to type.
    // Every line goes through `submit` and a real `step`, so what prints is the
    // world having actually run rather than a pose struck for the screenshot.
    let posed = requested_crossing();
    if request != "1" {
        match posed {
            // The last command is held back, the screen it was about to replace
            // is painted and kept, then it runs. Without this `ORBS_PASSAGE_AT`
            // has nothing to depart from and prints a settled screen while
            // looking as though it worked.
            Some(_) => {
                let (head, last) = split_last(request);
                drive(&mut sim, head);
                leaving(&sim, &screen, grid, &mut passing);
                drive(&mut sim, last);
            }
            None => drive(&mut sim, request),
        }
    }

    // Written after the script has run, because what a dump is *for* is the
    // world the script reached. There is no `quit` involved: a dump has no
    // session to leave, and `run_script` never reads `Quitting`.
    if asked && crate::save::write(&sim.snapshot()).is_err() {
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
        // Settled: a dump is a still, and a still of a pane halfway in is a
        // picture of a moment rather than of the screen.
        //
        // One pane at every grid, since the tower rail replaced the telemetry
        // pane. Asking `grid.fits(DEEP_FOCUS_FLOOR)` settled at two, which
        // after `PANES` became 1 drew a half-width session pane beside a second
        // one nothing painted. Whether the *rail* fits is
        // `ScreenLayout::compute`'s.
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
            drive(&mut sim, &after);
            // A `scribe` in `ORBS_THEN` opens the editor too, the only ordering
            // that shows a spell edited while it runs: the invocation has to be
            // cast before the editor opens on it.
            //
            // Opened, not typed into. `ORBS_EDIT` belongs to the `scribe` that
            // started it, and replaying it here types the script a second time
            // into a buffer that holds it — which it did, reporting a
            // three-line spell as nine. A save says nothing now (§19), so the
            // mistake would show up only as a doubled file under `peruse`.
            editing = editing.or_else(|| open(&mut sim));
            weaving = weaving.or_else(|| woven(&mut sim));
            walking |= walked(&mut sim);
            menuing = menuing.or_else(|| menued(&mut sim, &settings, &mut opened_manual));
        }
        // A maze can close from under the walker, and `walked` only latches
        // *on*. Both frontends give the keyboard back when the maze goes, so
        // without this a dump whose spell solved the maze draws `wander` still
        // owning the pane. Same divergence as the guide above, one surface
        // over.
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
            screen.refresh(
                sim.experience(),
                sim.renown(),
                sim.scale(),
                sim.ley_line(),
                sim.mastery(),
            );
        }
        // The running-line marker, and how the orb reads the buffer. The game
        // pushes both in through `editing::autosave`; a dump advances no
        // `Time`, so they come from the same accessors here — the panel below's
        // reason.
        //
        // The reading especially: without it `interpret` draws an empty page
        // and the marks never appear, a See-it line that looks like it works.
        //
        // And the guide, one accessor along. `Editor::new` seeds an empty
        // `Guide` for a frontend's first `refresh` to replace, and a dump does
        // none, so every dumped editor drew an empty box still taking thirty
        // columns off the buffer — a pane present, sized and blank is exactly
        // the dump-versus-game divergence `opened` exists to prevent.
        if let Some(editor) = editing.as_mut() {
            editor.set_running_line(sim.running_line(editor.name()));
            // Through the scrivener, so a capture can show a read line. The
            // reading is what `interpret` draws, and a dump that never consulted
            // one would be blind to the whole surface — CLAUDE.md's *"the
            // blindness looks exactly like stability"*.
            editor.set_reading(match super::scrivener() {
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
                editing: editing.as_mut(),
                weaving: weaving.as_ref(),
                walking,
                menuing: menuing.as_ref(),
                reading_manual: reading_manual.as_mut(),
            },
        );
    } else {
        super::prompt::paint_too_small(&mut frame, &sim);
    }

    print(&frame);
}

/// A wash's colour, as a writer would name it.
///
/// Two names joined for the flask's mixing band, the one region that is two
/// materials at once; printing only the first would hide what its picture is
/// about.
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

    // A tint is pure colour, so the frame above cannot show it — the glyphs are
    // identical either way. Without this a material whose colour never reaches
    // a cell draws in the base hue and looks exactly like one nobody has
    // tinted.
    //
    // Printed only when there are some, so every other dump is unchanged.
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

    // And the spell's parts of speech, for the tints' reason: a hue changes no
    // glyph, so a classified lexeme whose colour never reaches a cell looks
    // like a word nobody has coloured. `ink` sees this in the terminal build
    // and nothing sees it in the Bevy one, so this is the only text gate there
    // is.
    //
    // Runs in reading order rather than a tally: a word classified wrongly is
    // visible only beside the words either side of it.
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
/// A trailing `:<fraction>` names how far through, so `ORBS_BOOT=post:1` is the
/// finished card and bare `post` still means 0.5. The card's last line needed
/// it: at 0.5 only the studio has landed, so the two version lines were
/// unreachable from any switch — and the engine line is the one thing on that
/// card that differs between the frontends.
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
/// The plain session, deliberately: the screen being *left* is the one before
/// the last command ran, and that command is what opens a surface, so a
/// departing screen with an editor over it is the wrong moment. `ORBS_EDIT`,
/// `ORBS_THEN` and the rest belong to the arriving screen.
fn leaving(
    sim: &Sim,
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
            editing: None,
            weaving: None,
            walking: false,
            menuing: None,
            reading_manual: None,
        },
    );
    passing.pose_kept(&frame);
}

/// The shape and fraction `ORBS_PASSAGE_AT` asks for, if it asks.
///
/// Takes `requested_stage`'s shape — a bare fraction, or `name:fraction` — so a
/// second one need not be learned. A bare fraction poses the shape the game
/// would have chosen: `ORBS_PASSAGE_AT=0.3` is a `Wipe` on a room change and a
/// `Gather` on a `wander`. Naming a shape overrides that, which is how one is
/// looked at somewhere it does not normally run.
///
/// A malformed value poses nothing, for the reason a malformed grid falls back:
/// the useful answer to a typo is the ordinary screen. An unknown *name* poses
/// nothing either, since a dump that ignores half of what it was asked is worse
/// than one that does nothing.
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
/// Four switches read the same shape and had four copies of the parse. The
/// `is_finite` check is the part worth having in one place: a `NaN` survives
/// `clamp`, and a `NaN` phase makes `pulse` draw a still picture that looks
/// like a broken animation rather than a typo.
fn number(name: &str) -> Option<f32> {
    let value = std::env::var(name).ok()?;
    let parsed = value.trim().parse::<f32>().ok()?;
    parsed.is_finite().then_some(parsed)
}

/// The athanor's fire, at whatever phase `ORBS_FIRE_PHASE` asks for.
///
/// A dump advances no `Time`, so without this the fire draws at phase zero for
/// ever and "see it" degrades to "it compiles". Stepping the value shows the
/// plume move:
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
        // `page` moves by records and clamps to a total; asking it for exactly
        // the requested distance, with that distance as the ceiling, lands on it.
        let back = usize::from(back);
        scroll.page(back, true, back);
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
/// One loop rather than two, so the commands before an editing session and
/// after it are driven identically — a second copy would be a second answer to
/// what a dump command *is*.
fn drive(sim: &mut Sim, script: &str) {
    // Read once, for the whole script. `ORBS_AUGURY` is off unless asked for,
    // so an older dump drives as it always did, and one with `stub` set reaches
    // a divined line — the only way `dumps.sh` sees that surface at all.
    let augury = super::environment::augury();
    for line in script
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        match augury.as_deref() {
            Some(augur) => sim.submit_reading(line, augur),
            None => sim.submit(line),
        }
        sim.step();
    }
}

/// The editor a pending `scribe` asked for, with nothing typed into it.
///
/// The same two lines `editing::open_requested` runs in the game. Separate from
/// [`opened`] because `ORBS_EDIT` is one session belonging to one `scribe`, and
/// a second `scribe` later in the dump must open a buffer rather than replay it.
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
/// A dump presses no keys, so the keystrokes replay through the same
/// [`Editor`](super::Editor) methods the shell's key handler calls — a second
/// implementation would let dump and game disagree about what typing does.
///
/// The `:` goes through `insert` like any other character rather than into its
/// innards, because that is where the editor's two states decide whether a
/// keystroke is a word or a line — and it went wrong once: under the old `:`
/// command line, opening it required the caret at column 0, so saving after
/// typing a line put a colon in the spell, and reaching past that function hid
/// it from both this and the unit tests.
///
/// No marker for a command, because the editor's state already says which a
/// segment is: it opens in `Mode::Command`, `edit` switches to the buffer,
/// `<esc>` switches back. The script is the keystrokes in order.
///
/// A `save` here writes for real — `Sim::write_spell` queues the write and the
/// dump steps afterwards — so the picture is of a spell that has been saved.
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
/// Through `Sim::walk`, the door a key press goes through, for [`opened`]'s
/// reason: a second implementation would let dump and game disagree about how
/// much of the world moved, since walking costs no tick and `submit`/`step`
/// would cost one an arrow.
///
/// Returns whether the arrows ended up with the maze, which is what the status
/// row draws from.
fn walked(sim: &mut orbs_sim::Sim) -> bool {
    if !sim.wandering() {
        return false;
    }
    let Ok(script) = std::env::var(WALK) else {
        return true;
    };
    for segment in script.replace("\\n", "\n").split('\n') {
        // The four tokens a keyboard has and a shell word does not.
        let way = match segment.trim() {
            "" => continue,
            "<up>" => orbs_sim::tower::Way::North,
            "<right>" => orbs_sim::tower::Way::East,
            "<down>" => orbs_sim::tower::Way::South,
            "<left>" => orbs_sim::tower::Way::West,
            // Anything else ends the walk, which is what Escape does.
            _ => return false,
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

/// Re-read the tracks from the world.
///
/// A function rather than a closure over `sim`, so [`woven`] can hand a taken
/// node back: a closure capturing `&sim` holds an immutable borrow for its
/// whole life and `Sim::take` needs a mutable one.
fn refresh(screen: &mut super::Tapestry, sim: &orbs_sim::Sim) {
    screen.refresh(
        sim.experience(),
        sim.renown(),
        sim.scale(),
        sim.ley_line(),
        sim.mastery(),
    );
}

/// Save the buffer, through whatever scrivener the environment named.
///
/// One place, so the two save words cannot differ: `w` and `wq` both reach
/// this, and a reader consulted by one and not the other would make the spell
/// depend on how it was closed.
fn wrote(sim: &mut orbs_sim::Sim, editor: &super::Editor) {
    match super::scrivener() {
        Some(reader) => sim.write_spell_reading(editor.name(), editor.lines(), reader.as_ref()),
        None => sim.write_spell(editor.name(), editor.lines()),
    }
}

/// The orb's menu, if a `quit` in the script opened it, with [`MENU`] typed in.
///
/// `sim.menuing()` takes the handshake, as `woven` takes `weaving`, so this
/// fires once per `menu` and a second `menu` in `ORBS_THEN` reopens a screen
/// the first one's `resume` closed — the game's behaviour.
fn menued(
    sim: &mut orbs_sim::Sim,
    settings: &[crate::settings::Row],
    opened: &mut bool,
) -> Option<super::Menu> {
    // `ORBS_THRESHOLD=1` opens it without a word, because at the threshold
    // there is no prompt to type one at, and a dump has no `Threshold` resource
    // to read. The stance is what the capture is *of*: the threshold's top page
    // does not offer `resume` and `<esc>` does not close it, so the wrong
    // stance shows a screen no player sees.
    let stance = crate::Threshold::chosen(crate::Threshold::Playing);
    let stance = if stance.is_waiting() {
        crate::Stance::Threshold
    } else {
        crate::Stance::InTower
    };
    if stance == crate::Stance::InTower && !sim.menuing() {
        return None;
    }
    let mut menu = super::Menu::at(stance);
    // The caller's, for the engine line's reason: a dump cannot ask a world
    // what the tube is set to, and the frontends do not share settings. Each
    // binary hands over what it would show on a first launch, which is also
    // what makes the capture reproducible: `ORBS_SAVE=off` means there is no
    // file to read.
    menu.show_settings(settings.to_vec());

    let Ok(script) = std::env::var(MENU) else {
        return Some(menu);
    };
    for segment in script.replace("\\n", "\n").split('\n') {
        match segment.trim() {
            "" => continue,
            "<esc>" => {
                if menu.escape().is_some() {
                    return None;
                }
            }
            word => {
                for character in word.chars() {
                    menu.type_text(&character.to_string());
                }
                // Enter is implied at the end of a segment, as it is for the
                // weave: there is no buffer here, so a segment can only mean
                // "a word, now run it".
                match menu.enter() {
                    // `resume` really closes it, so `ORBS_MENU="resume"` draws
                    // the tower — the half of the See-it line that proves the
                    // menu is a place you can leave.
                    Some(super::MenuOutcome::Close) => return None,
                    // The manual really opens, unlike the outcomes below: it
                    // needs only the book. `opened` is where the reader ends
                    // up; `menued` returns the menu it is over.
                    Some(super::MenuOutcome::OpenManual) => *opened = true,
                    // Nothing to end and nothing to swap — see [`MENU`]. A dump
                    // hosts no process, so putting the orb down, loading
                    // another tower and beginning one all leave the menu on
                    // screen as the last thing the player saw. The *pages* are
                    // what a dump can show: `ORBS_MENU="saves"` is the
                    // listing's See-it line. `Drive` included, and it already
                    // happened — the menu writes the setting itself, and all
                    // this carries is *tell the running frontend*, which a dump
                    // is not.
                    Some(
                        super::MenuOutcome::Set { .. }
                        | super::MenuOutcome::PutDown
                        | super::MenuOutcome::Load(_)
                        | super::MenuOutcome::Begin { .. }
                        | super::MenuOutcome::Drive(_),
                    )
                    | None => {}
                }
            }
        }
    }
    Some(menu)
}

/// The manual a `manual` in `ORBS_MENU` opened, driven by `ORBS_MANUAL`.
///
/// The reader really runs, unlike the menu's other outcomes: it needs only the
/// book, so a dump shows what a player sees. `<esc>` from the contents closes
/// it and the menu behind is drawn — the half of the See-it line that proves it
/// is a place you can leave.
fn read_manual(sim: &orbs_sim::Sim, opened: bool) -> Option<super::ManualReader> {
    if !opened {
        return None;
    }
    let mut reader = super::ManualReader::of(super::manual_book(sim));
    let Ok(script) = std::env::var(MANUAL) else {
        return Some(reader);
    };
    for segment in script.replace("\\n", "\n").split('\n') {
        let key = match segment.trim() {
            "" => continue,
            "<esc>" => super::Key::Escape,
            "<pgdn>" => super::Key::PageDown,
            "<pgup>" => super::Key::PageUp,
            "<down>" => super::Key::Down,
            "<up>" => super::Key::Up,
            word => {
                for glyph in word.chars() {
                    reader.type_text(&glyph.to_string());
                }
                super::Key::Enter
            }
        };
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
/// Through the same `Tapestry` methods the key handler calls, for [`opened`]'s
/// reason. The reading is pushed in first and again at the end — the game's
/// `weaving::refresh` does it every frame, and a dump builds no `App`, so the
/// screen would otherwise draw a tapestry with no tracks for the words to point
/// at.
fn woven(sim: &mut orbs_sim::Sim) -> Option<super::Tapestry> {
    if !sim.weaving() {
        return None;
    }
    let mut screen = super::Tapestry::default();
    refresh(&mut screen, sim);

    let Ok(script) = std::env::var(WEAVE) else {
        return Some(screen);
    };
    for segment in script.replace("\\n", "\n").split('\n') {
        match segment.trim() {
            "" => continue,
            // The five tokens a keyboard has and a shell word does not.
            "<esc>" => screen.escape(),
            "<up>" => screen.step(0, -1),
            "<down>" => screen.step(0, 1),
            "<left>" => screen.step(-1, 0),
            "<right>" => screen.step(1, 0),
            word => {
                for character in word.chars() {
                    screen.type_text(&character.to_string());
                }
                // Enter is implied at the end of a segment and nowhere else:
                // there is no buffer here, so unlike the editor no state makes
                // a segment anything but "a word, now run it".
                match screen.enter() {
                    Some(super::WeaveOutcome::Close) => return None,
                    // The dump takes it too, or `ORBS_WEAVE="mastery\ntake"`
                    // would draw a screen where nothing happened — the See-it
                    // line for the whole progression tree. It queues, and
                    // `ORBS_THEN` is where the granting tick comes from.
                    Some(super::WeaveOutcome::Take(id)) => sim.take(&id),
                    None => {}
                }
            }
        }
    }
    refresh(&mut screen, sim);
    Some(screen)
}

/// The grid to draw into, from `ORBS_GRID` or §4's floor.
///
/// A malformed value falls back rather than panicking: this is a development
/// switch, and the useful answer to a typo is the default screen plus the
/// obvious mismatch, not a stack trace.
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
