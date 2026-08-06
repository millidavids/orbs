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

use orbs_render::{DisplayMode, Fidelity, Frame, GridSize};
use orbs_sim::Sim;

use super::line::Line;
use super::linear::Linear;
use super::screen::Screen;
use super::transition::PaneTransition;
use crate::boot::Stage;

/// The variable that asks for a dump, and optionally what to type first.
const DUMP: &str = "ORBS_DUMP";

/// The variable that overrides the grid, as `COLSxROWS`.
const GRID: &str = "ORBS_GRID";

/// The variable that picks a boot stage to dump.
const BOOT: &str = "ORBS_BOOT";

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

/// Commands are separated by this, so one shell word can drive a session.
const SEPARATOR: char = ';';

/// The grid a dump uses unless asked otherwise: §4's floor, where everything is
/// tightest and a layout bug shows first.
const DEFAULT_GRID: GridSize = GridSize { cols: 80, rows: 22 };

/// Draw one frame as text if `ORBS_DUMP` asked for it.
///
/// Returns whether it did, so `main` can exit instead of opening a window.
/// Called before the `App` is built: the point is to need none of it.
pub(crate) fn run(seed: u64, wizard: Option<String>) -> bool {
    let Ok(request) = std::env::var(DUMP) else {
        return false;
    };

    let mut sim = Sim::new(seed);
    if let Some(name) = wizard {
        sim.rename(&name);
    }
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6). A dump
    // builds no `App` and so has no watcher, but it must still read what is on
    // disk — otherwise the one tool CLAUDE.md says to reach for first is the one
    // tool that cannot show a writer their own edit.
    if std::env::var_os(crate::sim::content::CONTENT_DIR).is_some() {
        match crate::sim::content::load() {
            Some(prose) => sim.set_prose(prose),
            // A dump installs no `tracing` subscriber, so `content`'s own
            // warning goes nowhere. Without this line a writer with malformed
            // TOML sees their edit quietly not happen, which is the exact
            // failure mode the built-in fallback otherwise looks like.
            None => eprintln!(
                "warn: {} is set but no prose loaded; drawing the built-in text",
                crate::sim::content::CONTENT_DIR
            ),
        }
    }

    // `1` is the idiom for "just boot it"; anything else is a session to type.
    // Every line goes through `submit` and a real `step`, so what prints is the
    // world having actually run rather than a pose struck for the screenshot.
    if request != "1" {
        drive(&mut sim, &request);
    }

    let grid = grid();
    let screen = Screen {
        // **The tier the grid implies, not a fixed one.** This pinned
        // `tier_one((1280, 720))` under a comment claiming the tier was inert —
        // and it stopped being inert the moment the prompt learned to take two
        // rows at a fine tier, because `paint` reads `Fidelity::input_rows`. So
        // every dump reserved a magnified prompt, halved the input viewport, and
        // labelled itself with a tier its `ORBS_GRID` could never produce: a tool
        // CLAUDE.md sells as drawing "the same frame the game draws" was drawing
        // a combination the game cannot reach, which hides exactly the layout
        // bugs it exists to find.
        fidelity: Fidelity::for_grid(grid),
        grid,
        mode: DisplayMode::default_for(grid),
    };

    let mut frame = Frame::new(grid);
    // `ORBS_BOOT=frame` dumps that stage instead of the game. Boot runs once per
    // launch and the window will not composite in a detached shell, so without
    // this the sequence could only be checked by a person sitting in front of it
    // — which is exactly the position `ORBS_DUMP` exists to get out of.
    if let Some((stage, progress)) = requested_stage() {
        super::prompt::paint_booting(&mut frame, &screen, stage, progress);
        print(&frame);
        return true;
    }
    // The same branch `render::redraw` takes, and for the same reason: below the
    // floor the game draws a "too small" screen rather than a mangled layout, and
    // a tool documented as drawing the same frame the game draws has to draw that
    // one too. `ORBS_GRID=40x10` is how anyone would ever look at it.
    if screen.is_hostable() {
        // Settled: a dump is a still, and a still of a pane halfway in would be
        // a picture of a moment rather than of the screen.
        let panes = PaneTransition::settled(if grid.fits(orbs_render::DEEP_FOCUS_FLOOR) {
            2
        } else {
            1
        });
        let typed = std::env::var(LINE).map_or_else(|_| Line::default(), |text| Line::typed(&text));
        // If a `scribe` in `ORBS_DUMP` asked for the editor, open it — and let
        // `ORBS_EDIT` type into it. Without this the one surface the whole item
        // is about could only be looked at by a person sitting in front of a
        // window, which is the position `ORBS_DUMP` exists to get out of.
        let mut editing = opened(&mut sim);
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
            // time, and the dump reported `9 lines, 1 the orb could not read`
            // for a three-line spell.
            editing = editing.or_else(|| open(&mut sim));
        }
        // The running-line marker. In the game this is pushed in each frame by
        // `editing::autosave`; a dump builds no `App` and advances no `Time`, so
        // it is done here from the same accessor — the same reason the panel
        // below is computed rather than left empty.
        if let Some(editor) = editing.as_mut() {
            editor.set_running_line(sim.running_line(editor.name()));
        }
        // A dump builds no `App`, so the two cached resources have nobody to
        // fill them: they are computed here from the same functions the systems
        // call, rather than left empty — a dump that silently omitted the panel
        // would be a picture that proves the wrong thing.
        // Taken before the borrow below, because `unfurling` takes rather than
        // reads and `View` holds `&sim` for the whole call.
        let scroll = scrolled(&mut sim);
        let panel = super::input::Panel {
            instruments: sim.instruments(),
            domain: orbs_sim::parser::leaf(&sim.location()).to_owned(),
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
                offered: &super::input::Offered::default(),
                scroll: &scroll,
                ghost: &typed.ghost(sim.scene(), !sim.choices().is_empty()),
                panel: &panel,
                editing: editing.as_mut(),
            },
        );
    } else {
        super::prompt::paint_too_small(&mut frame, &sim);
    }

    print(&frame);
    true
}

/// The frame, then what it says.
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
}

/// The boot stage `ORBS_BOOT` asked for, and how far through it.
///
/// `ORBS_BOOT=frame` is halfway through that stage — halfway is the only
/// interesting point for a stage that animates, and naming a fraction as well
/// would be a switch with two knobs nobody turns. `ORBS_BOOT=0` is the
/// skip-it-entirely case and belongs to the running game, not here.
fn requested_stage() -> Option<(Stage, f32)> {
    let request = std::env::var(BOOT).ok()?;
    // `frame` outlived the stage it named. The border and the card are one stage
    // now, so it selects the moment the box is still closing and the first
    // letter is landing — which is what anyone typing `frame` wanted to look at,
    // and was never a thing the old stage could show.
    let (stage, progress) = match request.as_str() {
        "dark" => (Stage::Dark, 0.5),
        "frame" => (Stage::Post, Stage::FRAME_SHARE / 2.0),
        "post" => (Stage::Post, 0.5),
        _ => return None,
    };
    Some((stage, progress))
}

/// The grid to draw into, from `ORBS_GRID` or §4's floor.
///
/// A malformed value falls back rather than panicking: this is a development
/// switch, and the useful answer to a typo is the default screen plus the
/// obvious mismatch, not a stack trace.
/// How far back `ORBS_SCROLL` asks the transcript to be.
///
/// A malformed value scrolls nowhere, for the same reason a malformed grid falls
/// back: the useful answer to a typo is the default screen, not a stack trace.
fn scrolled(sim: &mut orbs_sim::Sim) -> super::input::Scroll {
    let mut scroll = super::input::Scroll::default();
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
