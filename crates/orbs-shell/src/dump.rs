//! The game's own screen, as text, with no window and no GPU.
//!
//! CLAUDE.md's working practice says work is not done when it compiles but when
//! it has been *looked at*, and `ORBS_CAPTURE=1` was how. That path needs a
//! composited window: run the binary from a detached shell, or with the display
//! asleep, and the screenshot is a valid PNG of a black rectangle. The renderer
//! is fine and the picture proves nothing — which is worse than no picture,
//! because it looks like evidence.
//!
//! So this draws the **same frame the game draws** — [`super::paint`], the real
//! [`Sim`], the real [`ScreenLayout`](orbs_render::ScreenLayout) — into a
//! [`Frame`] nobody rasterises, and prints it. No `App`, no `DefaultPlugins`, no
//! adapter. What it cannot show is the parts a frontend owns and the Frame does
//! not: phosphor, the CRT curve, the blinking caret. Those still need eyes on a
//! window, and rule 2 is exactly the promise that nothing *informational* is
//! among them.
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
/// Named for the fire because that is what it was built for; it drives the
/// mortar's stroke and everything after it too. Renaming an environment variable
/// that is written down in CLAUDE.md and §15's See-it lines costs more than the
/// slight misnomer does.
const FIRE_PHASE: &str = "ORBS_FIRE_PHASE";

/// How far through the current **world tick** to draw, `0.0`..`1.0`.
///
/// A dump builds no `App`, so there is no `Time<Fixed>` to read the position
/// from and every bar would sit exactly on a tick boundary — which is precisely
/// the jump the creep exists to remove, making it the one thing about it a dump
/// could not show. See [`bench()`].
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

/// A line left **unsubmitted** in the prompt.
///
/// `ORBS_DUMP` submits every `;`-separated segment, so the input buffer is
/// always empty by the time the frame is painted — which makes a caret position,
/// a partly-typed word and a suggestion ghost the three things a dump cannot
/// show. Everything the prompt does between keystrokes needed this to be gated
/// at all.
///
/// ```text
/// ORBS_DUMP="attend laboratory" ORBS_LINE="wield mo" cargo run -p orbs
/// ```
const LINE: &str = "ORBS_LINE";

/// How many records to hold back from the newest end.
///
/// The transcript's scroll is a keypress, and a dump presses no keys — so
/// without this the one thing §15 asks for, *reaching it from the running game*,
/// could only be checked by a person sitting in front of a window. Same reason
/// `ORBS_LINE` exists.
const SCROLL: &str = "ORBS_SCROLL";

/// What to type into the spell editor, once `scribe` has opened it.
///
/// Newline-separated keystrokes, in order. **The editor's own state decides what
/// a segment is** — it opens in command state, so the first segment is a word,
/// `edit` drops into the buffer, and the token `<esc>` comes back out:
///
/// ```text
/// ORBS_DUMP="scribe morning" \
///   ORBS_EDIT="edit\ngrind sage\n<esc>\nquit" cargo run -p orbs
/// ```
///
/// **`quit` is how a dump saves**, and there is no `save` to reach for. In the
/// running game the buffer writes itself out a beat after the typing stops, and
/// that pause is measured off `Time` — which a dump does not advance, having no
/// frames. `quit` flushes, which is why it is the last segment above; `w` and
/// `wq` also work, and write without closing and with closing respectively.
///
/// Without this the editor could only be looked at by a person sitting in front
/// of a window, and it is the surface this whole item is about. Same reason
/// `ORBS_LINE` and `ORBS_SCROLL` exist.
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
/// `\n`-separated, and **every segment is a whole thing** — a word, or one of
/// the arrow tokens. Unlike `ORBS_EDIT` there is no buffer, so nothing is typed a
/// character at a time and no Enter is ever implied between segments: a word runs
/// when its segment ends, and an arrow moves the cursor and runs nothing.
///
/// **The arrows do nothing until a word has gone into a track**, exactly as they
/// do in the game — `ley` or `mastery` is what hands them over, the way `edit`
/// drops into the editor's buffer.
///
/// ```text
/// ORBS_DUMP="weave" ORBS_WEAVE="mastery\n<right>\ntake" cargo run -p orbs
/// ```
const WEAVE: &str = "ORBS_WEAVE";

/// Arrow presses for the stacks a `wander` in `ORBS_DUMP` took the keys for.
///
/// `\n`-separated, one of `<up>`, `<right>`, `<down>`, `<left>` per segment.
/// Anything else ends the walk, the way Escape does.
///
/// **No tick per token**, because there is none in the game either: an arrow
/// goes through `Sim::walk`, which moves the reading and advances no clock. A
/// dump that stepped between presses would show a world eight seconds older than
/// the one a player would be looking at.
///
/// ```text
/// ORBS_DUMP="attend archive; divine; wander" ORBS_WALK="<right>\n<down>" cargo run -p orbs
/// ```
const WALK: &str = "ORBS_WALK";

/// Commands are separated by this, so one shell word can drive a session.
const SEPARATOR: char = ';';

/// The grid a dump uses unless asked otherwise: **the one the game draws**.
///
/// This was §4's 80×22 floor, on the grounds that a layout bug shows first where
/// everything is tightest. That reasoning was sound while the grid followed the
/// window and the floor was a screen the game could genuinely be at; now the
/// game is 120×45 always, and a dump showing anything else is an instrument
/// reading a screen nobody has. `ORBS_GRID=80x22` is the floor check, and is
/// what CLAUDE.md's See-it lines use when width is the thing under test.
const DEFAULT_GRID: GridSize = orbs_render::GRID;

/// Draw one frame as text if `ORBS_DUMP` asked for it.
///
/// Returns whether it did, so a frontend can exit instead of opening a window.
/// Called before the `App` is built, or before raw mode is entered: the point is
/// to need none of it.
///
/// `engine` is the POST card's third line — the one fact only the caller knows.
/// See the POST card's own module for why it cannot be a constant.
#[must_use]
pub fn run(seed: u64, wizard: Option<String>, engine: &str) -> bool {
    let Some(request) = requested() else {
        return false;
    };
    run_script(seed, wizard, engine, &request);
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
/// **This is the boundary proof.** Both binaries reach it, through the same
/// painters, from the same `Sim` — so `orbs-tui --dump X` and `ORBS_DUMP=X orbs`
/// print the same bytes or the shell has grown a frontend-shaped hole in it.
pub fn run_script(seed: u64, wizard: Option<String>, engine: &str, request: &str) {
    // **A dump neither loads nor saves unless `ORBS_SAVE` names a path.**
    //
    // Not a convenience — the alternative breaks the instruments. A dump that
    // loaded by default would make every See-it line in CLAUDE.md depend on
    // whether anyone had played in that directory, and `scripts/dumps.sh` would
    // stop being a baseline the first time one of its 56 screens wrote a save
    // the next 55 then read. `orbs-save.toml` sitting in the repository root
    // would silently change what a dump draws.
    //
    // So the default here is the opposite of the running game's: no file at all,
    // and a path only when a person asks for one by name.
    let asked = std::env::var_os(crate::save::SAVE_VAR).is_some();
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
            let mut fresh = Sim::new(seed);
            if let Some(name) = wizard {
                fresh.rename(&name);
            }
            fresh.say_save_unreadable();
            fresh
        }
        crate::save::Opened::New => {
            let mut fresh = Sim::new(seed);
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
    // builds no `App` and so has no watcher, but it must still read what is on
    // disk — otherwise the one tool CLAUDE.md says to reach for first is the one
    // tool that cannot show a writer their own edit.
    if std::env::var_os(crate::prose::CONTENT_DIR).is_some() {
        match crate::prose::load() {
            Some(prose) => sim.set_prose(prose),
            // A dump installs no `tracing` subscriber, so `content`'s own
            // warning goes nowhere. Without this line a writer with malformed
            // TOML sees their edit quietly not happen, which is the exact
            // failure mode the built-in fallback otherwise looks like.
            None => eprintln!(
                "warn: {} is set but no prose loaded; drawing the built-in text",
                crate::prose::CONTENT_DIR
            ),
        }
    }

    // `1` is the idiom for "just boot it"; anything else is a session to type.
    // Every line goes through `submit` and a real `step`, so what prints is the
    // world having actually run rather than a pose struck for the screenshot.
    if request != "1" {
        drive(&mut sim, request);
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

    let grid = grid();
    // **Wide, which is what every running frontend opens in.** This derived the
    // mode from the grid instead — the only call to `DisplayMode::default_for`
    // in the workspace — so the project's primary See-it instrument drew the
    // *opposite* focus mode to the game it is the instrument for: a dump printed
    // `focus deep` and `F4 wide` where the running terminal printed `focus wide`
    // and `F4 deep`.
    //
    // It is only the two labels today, because one pane makes both tilings
    // identical and §19 records `F4` as visibly inert until Phase 9a returns the
    // second pane. When it does, every dump would have shown a layout the game
    // never draws — and CLAUDE.md's own See-it blocks quote this output. Its
    // rule for exactly this: *"a See-it line that describes a different screen
    // is worse than none."*
    let screen = Screen::windowless(grid, None);

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
        // Settled: a dump is a still, and a still of a pane halfway in would be
        // a picture of a moment rather than of the screen.
        //
        // **One pane, at every grid**, since the tower rail replaced the
        // telemetry pane. This asked `grid.fits(DEEP_FOCUS_FLOOR)` and settled
        // at two — which after `PANES` became 1 left the dump drawing a screen
        // the game does not have: a half-width session pane beside a second one
        // nothing painted. Whether the *rail* fits is `ScreenLayout::compute`'s
        // decision and is taken from the grid there, so there is nothing left
        // for this branch to ask.
        let panes = PaneTransition::settled(1);
        let typed = std::env::var(LINE).map_or_else(|_| Line::default(), |text| Line::typed(&text));
        // If a `scribe` in `ORBS_DUMP` asked for the editor, open it — and let
        // `ORBS_EDIT` type into it. Without this the one surface the whole item
        // is about could only be looked at by a person sitting in front of a
        // window, which is the position `ORBS_DUMP` exists to get out of.
        let mut editing = opened(&mut sim);
        // ...and the same for a `weave`. Taken before `ORBS_THEN` runs, so a
        // dump can open the screen and then keep issuing commands behind it —
        // which is what the world ticking behind a modal surface looks like.
        let mut weaving = woven(&mut sim);
        // ...and the same for a `wander`, except that this one owns no surface,
        // so what it produces is a flag and some steps already walked.
        let mut walking = walked(&mut sim);
        // Commands to run *after* the editing session. A `:w` queues its write
        // for the next tick like every other effect, so a `peruse` typed in
        // `ORBS_DUMP` runs before the spell exists — it would offer the other
        // readables instead, which looks exactly like a bug and is not one.
        // This is the only way to look at what was just saved.
        if let Ok(after) = std::env::var(THEN) {
            sim.step();
            drive(&mut sim, &after);
            // A `scribe` **in `ORBS_THEN`** opens the editor too, and that is the
            // only ordering that can show a spell being edited while it runs:
            // the invocation has to be cast before the editor is opened on it.
            //
            // **Opened, not typed into.** `ORBS_EDIT` has had its session by now
            // and belongs to the `scribe` that started it; replaying it here
            // types the whole script a second time into a buffer that already
            // holds it. That is not hypothetical — it is what this did first
            // time, and the dump reported a three-line spell as nine lines with
            // one the orb could not read. **A save says nothing now** (§19), so
            // the same mistake would show up only as a doubled file under
            // `peruse` — which is why this comment outlived the message.
            editing = editing.or_else(|| open(&mut sim));
            weaving = weaving.or_else(|| woven(&mut sim));
            walking |= walked(&mut sim);
        }
        // **And a maze can close from under the walker.** `walked` only ever
        // latches *on*; both frontends give the keyboard back when the maze goes
        // — `Surfaces::tick`'s `if self.walking && sim.stacks().is_none()` — so
        // without this a dump whose spell solved the maze draws `wander` still
        // owning the whole pane, where the game has handed it back to the
        // prompt. Same divergence as the guide above, one surface over.
        if walking && sim.stacks().is_none() {
            walking = false;
        }
        // The world may have moved while the screen was up — `ORBS_THEN` steps.
        // In the game `weaving::refresh` runs every frame for exactly this.
        if let Some(screen) = weaving.as_mut() {
            screen.refresh(sim.experience(), sim.ley_line(), sim.mastery());
        }
        // The running-line marker, and how the orb reads the buffer. In the game
        // both are pushed in by `editing::autosave`; a dump builds no `App` and
        // advances no `Time`, so they are done here from the same accessors —
        // the same reason the panel below is computed rather than left empty.
        //
        // **The reading especially.** Without it `interpret` draws an empty page
        // and the marks never appear, which is a See-it line that looks like it
        // works and proves nothing — worse than no picture at all.
        //
        // **And the guide**, which is the same argument one accessor along.
        // `Editor::new` seeds an empty `Guide` for the first `refresh` a
        // frontend does to replace — Bevy in `editing::open_requested` and on
        // every key, `orbs-tui` in `Surfaces::refresh` — and a dump does
        // neither, so every dumped editor drew an empty box that still took
        // thirty columns off the buffer. `scripts/dumps.sh` captures several of
        // those, and this is the project's primary See-it instrument: a pane
        // that is present, sized, and blank is exactly the dump-versus-game
        // divergence `opened` exists to prevent.
        if let Some(editor) = editing.as_mut() {
            editor.set_running_line(sim.running_line(editor.name()));
            editor.set_reading(sim.read_spell(editor.domain(), editor.lines()));
            editor.refresh(&sim);
        }
        // A dump builds no `App`, so the two cached resources have nobody to
        // fill them: they are computed here from the same functions the systems
        // call, rather than left empty — a dump that silently omitted the panel
        // would be a picture that proves the wrong thing.
        // Taken before the borrow below, because `unfurling` takes rather than
        // reads and `View` holds `&sim` for the whole call.
        let scroll = scrolled(&mut sim);
        let panel = super::glance::Panel {
            instruments: sim.instruments(),
            domain: orbs_sim::parser::leaf(&sim.location()).to_owned(),
            stacks: sim.stacks(),
            ward: sim.ward(),
            pylon: sim.pylon(),
            briefs: sim.briefs(),
        };
        super::prompt::paint(
            &mut frame,
            &mut Linear::default(),
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
                // A dump builds no `App` and advances no `Time`, so every
                // instrument would draw at phase zero forever and the See-it
                // gate would be "it compiles". `ORBS_FIRE_PHASE` is what makes
                // the animations visible as text: step it by a tick and the
                // plume climbs, the pestle falls. See CLAUDE.md.
                bench: &bench(),
                editing: editing.as_mut(),
                weaving: weaving.as_ref(),
                walking,
            },
        );
    } else {
        super::prompt::paint_too_small(&mut frame, &sim);
    }

    print(&frame);
}

/// A wash's colour, as a writer would name it.
///
/// Two names joined for the flask's mixing band, which is the one region that is
/// two materials at once — and printing only the first would hide exactly the
/// thing that instrument's picture is about.
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

    // **A tint is pure colour, so the frame above cannot show it** — the glyphs
    // are identical with it and without. That makes it the one part of the panel
    // whose See-it line would otherwise be "it compiles", and the failure it
    // hides is total: a material reported by the sim whose colour never reaches a
    // cell draws in the base hue and looks exactly like a material nobody has
    // tinted yet.
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

    // **And the spell's parts of speech, for exactly the tints' reason.** A hue
    // changes no glyph, so a lexeme the sim classified whose colour never
    // reaches a cell draws in the base and looks precisely like a word nobody
    // has coloured yet. `ink` can see this in the terminal build and nothing can
    // see it in the Bevy one, which makes this the only text gate there is.
    //
    // Printed as *runs in reading order*, not as a tally: the interesting
    // failure is a word classified wrongly, and that is visible only beside the
    // words either side of it.
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
/// Spelled out rather than `{:?}`'d so the column reads as prose and so a
/// rename of the enum does not silently rewrite every captured baseline.
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
/// `ORBS_BOOT=frame` is halfway through that stage — halfway is the only
/// interesting point for a stage that animates. `ORBS_BOOT=0` is the
/// skip-it-entirely case and belongs to the running game, not here.
///
/// # `post:1` — because the card's last line had no See-it line at all
///
/// The report types itself: the studio, then the compiler, then **what the orb
/// is built out of**. At `post`'s 0.5 only the studio has landed, so the two
/// version lines were unreachable from any switch — and the engine line is the
/// one thing on that card that differs between the frontends, the whole reason
/// each build holds its own pin to its own manifest with a test.
///
/// A trailing `:<fraction>` names how far through, so `ORBS_BOOT=post:1` is the
/// finished card. Bare `post` still means 0.5 and every existing See-it line is
/// unchanged; this comment used to say a second knob would be one "nobody
/// turns", and it was wrong in the one place it mattered.
fn requested_stage() -> Option<(Stage, f32)> {
    let request = std::env::var(BOOT).ok()?;
    let (name, asked) = match request.split_once(':') {
        Some((name, fraction)) => (name, fraction.trim().parse::<f32>().ok()),
        None => (request.as_str(), None),
    };
    // `frame` outlived the stage it named. The border and the card are one stage
    // now, so it selects the moment the box is still closing and the first
    // letter is landing — which is what anyone typing `frame` wanted to look at,
    // and was never a thing the old stage could show.
    let (stage, progress) = match name {
        "dark" => (Stage::Dark, 0.5),
        "frame" => (Stage::Post, Stage::FRAME_SHARE / 2.0),
        "post" => (Stage::Post, 0.5),
        _ => return None,
    };
    Some((stage, asked.unwrap_or(progress).clamp(0.0, 1.0)))
}

/// A finite `f32` from the environment, if the variable holds one.
///
/// Four switches read the same shape — `ORBS_FIRE_PHASE`, `ORBS_FLARE`,
/// `ORBS_TICK`, `ORBS_LOAD` — and had four copies of the parse. The
/// `is_finite` check is the part worth having in one place: a `NaN` reaching
/// `clamp` comes back `NaN`, and a `NaN` phase makes every hash in `pulse` draw
/// from a wrapped-to-zero moment, which is a still picture that looks like a
/// broken animation rather than like a typo.
fn number(name: &str) -> Option<f32> {
    let value = std::env::var(name).ok()?;
    let parsed = value.trim().parse::<f32>().ok()?;
    parsed.is_finite().then_some(parsed)
}

/// The athanor's fire, at whatever phase `ORBS_FIRE_PHASE` asks for.
///
/// **A dump advances no `Time`**, so without this the fire draws at phase zero
/// forever and "see it" degrades to "it compiles". Stepping the value shows the
/// plume move:
///
/// ```text
/// ORBS_DUMP="attend laboratory; kindle charcoal; meditate 300" \
///   ORBS_FIRE_PHASE=0.4 cargo run -p orbs
/// ```
///
/// A malformed value burns at zero, for the same reason a malformed grid falls
/// back: the useful answer to a typo is the default screen, not a stack trace.
/// `ORBS_FIRE=0` still turns the effect off entirely — the two are separate
/// switches because a phase of zero is a perfectly ordinary phase.
fn bench() -> super::bench::Bench {
    let mut bench = super::bench::Bench::default();
    if let Some(phase) = number(FIRE_PHASE) {
        // A negative phase is refused by `tick` itself rather than here — see
        // `Bench::tick`, where running the decays backwards used to manufacture
        // an ignition and a pour out of nothing.
        bench.tick(phase);
    }
    // **After the phase**, because `tick` decays the flare — setting it first
    // would have the phase immediately burn it off, and `ORBS_FLARE=1` would
    // silently do nothing at any phase past a second.
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
    // An `unfurl` in the script hands the transcript the keyboard, exactly as
    // `plugin::start_reading` does in the game. A dump builds no `App`, so the
    // one system that would otherwise do this has nobody to run it — the same
    // reason the panel below is computed here rather than left empty.
    //
    // The page it lands on is `ORBS_SCROLL`'s, so the two compose: `ORBS_SCROLL`
    // says how far back to look and `unfurl` says the keys are live.
    if sim.unfurling() {
        scroll.read();
    }
    scroll
}

/// Submit every `;`-separated command in `script`, stepping between them.
///
/// One loop rather than two, so the commands before an editing session and the
/// ones after it are driven identically — a second copy would be a second answer
/// to what a dump command *is*.
fn drive(sim: &mut Sim, script: &str) {
    for line in script
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        sim.submit(line);
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
/// A dump presses no keys, so the keystrokes are replayed here through the same
/// [`Editor`](super::Editor) methods the shell's key handler calls — not through
/// a second implementation, which would let the dump and the game disagree about
/// what typing does.
///
/// **The `:` goes through `insert` like any other character**, rather than
/// calling into its innards. That looks like a detail and is not: the editor's
/// two states decide there whether a keystroke is a word or a line of a spell,
/// and a dump that skipped the decision could not show it going wrong. It did go
/// wrong once — under the old `:` command line, opening it required the caret at
/// column 0, so typing a line and then trying to save put a colon in the spell —
/// and neither this nor the unit tests could see it, because both reached past
/// the one function that had the bug.
///
/// **There is no marker for a command**, because the editor's own state already
/// says which a segment is: it opens in `Mode::Command`, so the first segment is
/// a word, `edit` switches to the buffer, and `<esc>` switches back. The script
/// is therefore the keystrokes in order and nothing else.
///
/// A `save` here **writes for real**: `Sim::write_spell` records the submission
/// and queues the write, and the dump steps afterwards. That is the point — the
/// picture is of a spell that has actually been saved.
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
                sim.write_spell(editor.name(), editor.lines());
                editor.saved();
            }
            Some(super::EditorOutcome::SaveAndClose) => {
                sim.write_spell(editor.name(), editor.lines());
                sim.step();
                return None;
            }
            None => {}
        }
    }
    Some(editor)
}

/// The arrow keys a `wander` in the dump asked for, with `ORBS_WALK` played into
/// them.
///
/// **Through `Sim::walk`, the same door a key press goes through**, for the
/// reason [`opened`] gives: a second implementation would let the dump and the
/// game disagree — here about how much of the world has moved, since walking
/// costs no tick and `submit`/`step` would cost one an arrow.
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
        // **`walk`, not `submit` and `step`.** The first version did the latter
        // and quietly showed a different game: a tick per arrow meant a dump of
        // eight presses had advanced the world eight seconds, so a brew could
        // finish and a fire burn down inside what is meant to be a still.
        if !sim.walk(way) {
            return false;
        }
    }
    true
}

/// The weave screen a `weave` in the dump asked for, with `ORBS_WEAVE` played
/// into it.
///
/// **Through the same `Tapestry` methods the key handler calls**, for the reason
/// [`opened`] gives: a second implementation would let the dump and the game
/// disagree about what a keystroke does.
///
/// The reading is pushed in first and again at the end. In the game
/// `weaving::refresh` does it every frame; a dump builds no `App`, so the screen
/// would otherwise draw a tapestry with no tracks in it — and the words below
/// need the tracks to have anything to point at.
fn woven(sim: &mut orbs_sim::Sim) -> Option<super::Tapestry> {
    if !sim.weaving() {
        return None;
    }
    let mut screen = super::Tapestry::default();
    let refresh = |screen: &mut super::Tapestry| {
        screen.refresh(sim.experience(), sim.ley_line(), sim.mastery());
    };
    refresh(&mut screen);

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
                // **Enter is implied at the end of a segment and nowhere else.**
                // There is no buffer here, so unlike the editor there is no state
                // in which a segment means anything but "a word, now run it".
                if screen.enter() == Some(super::WeaveOutcome::Close) {
                    return None;
                }
            }
        }
    }
    refresh(&mut screen);
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
