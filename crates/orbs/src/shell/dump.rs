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
//! ORBS_DUMP="attend alembic; decoct clarity; meditate 25" cargo run -p orbs
//! ORBS_DUMP=1 ORBS_GRID=120x33 cargo run -p orbs
//! ```

use orbs_render::{DisplayMode, Fidelity, Frame, GridSize};
use orbs_sim::Sim;

use super::input::Line;
use super::linear::Linear;
use super::screen::Screen;

/// The variable that asks for a dump, and optionally what to type first.
const DUMP: &str = "ORBS_DUMP";

/// The variable that overrides the grid, as `COLSxROWS`.
const GRID: &str = "ORBS_GRID";

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

    // `1` is the idiom for "just boot it"; anything else is a session to type.
    // Every line goes through `submit` and a real `step`, so what prints is the
    // world having actually run rather than a pose struck for the screenshot.
    if request != "1" {
        for line in request
            .split(SEPARATOR)
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            sim.submit(line);
            sim.step();
        }
    }

    let grid = grid();
    let screen = Screen {
        // A tier is a window-pixel fact and there is no window; the grid is
        // given directly, which is the only thing the layout reads.
        fidelity: Fidelity::tier_one((1280, 720)),
        grid,
        mode: DisplayMode::default_for(grid),
    };

    let mut frame = Frame::new(grid);
    // The same branch `render::redraw` takes, and for the same reason: below the
    // floor the game draws a "too small" screen rather than a mangled layout, and
    // a tool documented as drawing the same frame the game draws has to draw that
    // one too. `ORBS_GRID=40x10` is how anyone would ever look at it.
    if screen.is_hostable() {
        super::prompt::paint(
            &mut frame,
            &sim,
            &Line::default(),
            &screen,
            &mut Linear::default(),
        );
    } else {
        super::prompt::paint_too_small(&mut frame);
    }

    println!("{}", frame.to_text());
    println!("-- linearised (DESIGN.md §14) --");
    for utterance in frame.speech().utterances() {
        println!(
            "  {:<10} {}",
            format!("{:?}", utterance.kind),
            utterance.text
        );
    }
    true
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
