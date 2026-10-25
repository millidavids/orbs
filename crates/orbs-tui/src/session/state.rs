//! What a session holds, and how one is built.

use orbs_render::{Frame, GridSize};
use orbs_shell::{
    Bench, Boot, Climb, Line, Linear, Offered, PaneTransition, Panel, Passing, Reveal, Screen,
    Scroll, Threshold,
};
use orbs_sim::Sim;

use crate::blit;
use crate::carried::{Carried, settings};
use crate::keymap::HeldOver;
use crate::surfaces::Surfaces;

/// Everything the loop keeps between frames.
pub(crate) struct Session {
    pub(super) sim: Sim,
    /// The readers that answer what the orb cannot read itself (§6), if any.
    ///
    /// Built once with the session, not per keystroke: reading the environment
    /// is not free and the answer cannot change mid-run.
    pub(super) readers: orbs_shell::Readers,
    /// The player's settings, carried across a swap.
    ///
    /// The driver beside the readers, so switching back reads no file; the mode
    /// held, since `draw` rebuilds `Screen`. Its `linear` is unread:
    /// [`Session::carrying`] takes `linear`.
    pub(super) held: Carried,
    /// The file this tower is kept in, and where it is written back.
    ///
    /// Travels with the `Sim`, replaced together, as the Bevy build's `Kept`.
    /// `None` for `ORBS_SAVE=off`.
    pub(super) kept: Option<std::path::PathBuf>,
    /// Whether a failed write has already been complained about.
    pub(super) save_failed: bool,
    pub(super) line: Line,
    pub(super) offered: Offered,
    pub(super) ghost: String,
    pub(super) panel: Panel,
    pub(super) scroll: Scroll,
    pub(super) bench: Bench,
    pub(super) linear: Linear,
    pub(super) reveal: Reveal,
    pub(super) panes: PaneTransition,
    /// One screen leaving and the next arriving, on the `motion` row.
    pub(super) passing: Passing,
    /// The numbers rolling to the truth, on the same row.
    pub(super) climb: Climb,
    pub(super) frame: Frame,
    pub(super) screen: blit::Screen,
    pub(super) grid: GridSize,
    /// Whichever of the four has the keyboard, if any has.
    pub(super) surfaces: Surfaces,
    /// How many records the transcript last fitted, for `PageUp`.
    ///
    /// Measured by the painter, because only it knows how many records fit: a
    /// record is not one row, and paging by rows drops the lines in between.
    pub(super) page: usize,
    /// How far §4's boot sequence has got.
    ///
    /// The clock is `orbs-shell`'s and only the driving is here, the split the
    /// Bevy build has: `Boot::advance` walks the stages, `Stage` owns their
    /// durations, and a frontend supplies a `Time` and a frame. `Boot::default`
    /// reads `ORBS_BOOT=0` itself, so the skip works here for free.
    pub(super) boot: Boot,
    /// Whether a tower has been chosen yet.
    ///
    /// The same enum the Bevy build gates on, for `Boot`'s reason. At
    /// [`Threshold::Waiting`] the menu is up and cannot be closed, `tick` does
    /// not run and `kept` is `None`, so the scratch world cannot reach a file
    /// whatever else happens.
    pub(super) threshold: Threshold,
    pub(super) held_over: HeldOver,
    /// What this binary is built out of, for the card's third line.
    ///
    /// §4's card is a diegetic inventory of the machine, so it describes *this*
    /// one — `crossterm 0.29` where the other build says `bevy 0.20.0`. The
    /// frontend's to know and the shell's to place, as the wizard's name is.
    pub(super) engine: String,
    /// The installation, while `sim` is its practice tower.
    ///
    /// Set by a swap after the session is built, the one place a practice
    /// tower arrives; a new session for any other tower starts without one.
    pub(super) install: Option<orbs_shell::Install>,
    /// Whether §14's health warning is still to be put up — at the threshold,
    /// on the first frame that draws the menu.
    pub(super) warns: bool,
}

/// Nothing reading spells: `plain`, or a build with no weights.
///
/// Abstains on every line, so `Read` is `Held`. The sim's own default, named
/// here rather than branched around, so there is one write path instead of two.
static VERBATIM: orbs_sim::Verbatim = orbs_sim::Verbatim;

/// The spell reader in effect: what this build offers, if the player wants it,
/// and otherwise one that reads nothing. Fields rather than `&Session`, so a
/// caller can lend the sim mutably beside it.
pub(super) fn scribing(
    readers: &orbs_shell::Readers,
    driver: orbs_shell::Driver,
) -> &dyn orbs_sim::Scrivener {
    readers.scrivener(driver).unwrap_or(&VERBATIM)
}

impl Session {
    /// A session on this tower.
    ///
    /// `boot` a parameter, since a swap builds a new `Session` and
    /// `Boot::default()` would play the sequence a second time.
    pub(crate) fn new(
        sim: Sim,
        kept: Option<std::path::PathBuf>,
        boot: Boot,
        threshold: Threshold,
        carried: Carried,
        grid: GridSize,
        narrow: bool,
        engine: String,
    ) -> Self {
        let mut panel = Panel::default();
        panel.refresh(&sim);
        let mut linear = Linear::default();
        linear.show(carried.linear);
        let mut screen = blit::Screen::new(grid, narrow);
        screen.set_greyscale(carried.sight == orbs_shell::Sight::Greyscale);
        Self {
            boot,
            threshold,
            held_over: HeldOver::default(),
            engine,
            sim,
            readers: orbs_shell::Readers::from_environment(),
            kept,
            save_failed: false,
            line: Line::default(),
            offered: Offered::default(),
            ghost: String::new(),
            panel,
            scroll: Scroll::default(),
            bench: Bench::default(),
            linear,
            reveal: Reveal::default(),
            // A terminal has one pane and never animates between counts, so this
            // is settled from the first frame. `transition.rs`: *"`orbs-tui` is
            // free to ignore all of it."*
            panes: PaneTransition::settled(1),
            passing: Passing::default(),
            // Off until the first frame hands it the row; off is the truth.
            climb: Climb::default(),
            frame: Frame::new(grid),
            screen,
            grid,
            surfaces: Surfaces {
                // The threshold's menu goes up with the session: no word opens
                // it, there is no prompt to type one at, and it cannot be closed
                // (`Stance::Threshold`) — so this is the only place it goes up.
                // From the session, not hard-coded: passing `false` and `Wide`
                // let the Access page report `linear [off]` while the pane drew
                // the linear stream, §14's only visible control reporting the
                // opposite of what was in effect.
                menuing: threshold.is_waiting().then(|| {
                    orbs_shell::Menu::open_at(
                        orbs_shell::Stance::Threshold,
                        carried.driver,
                        settings(&carried),
                    )
                }),
                ..Surfaces::default()
            },
            // A sane floor until the first paint measures the real one.
            page: 1,
            install: None,
            warns: threshold.is_waiting(),
            held: carried,
        }
    }

    /// What this session hands to the next one.
    pub(super) fn carrying(&self) -> Carried {
        Carried {
            linear: self.linear.showing(),
            ..self.held.clone()
        }
    }

    /// Take on what `held` says is in effect.
    pub(super) fn hold(&mut self, held: Carried) {
        // Only on a change: `Linear::show` writes the file, so every other row
        // would otherwise rewrite this one too.
        if held.linear != self.linear.showing() {
            self.linear.show(held.linear);
        }
        self.screen
            .set_greyscale(held.sight == orbs_shell::Sight::Greyscale);
        self.held = held;
    }

    /// Whether the boot card has handed over.
    pub(crate) const fn booted(&self) -> bool {
        self.boot.is_live()
    }

    /// Whether the tower's clock runs: not during boot, and not at the threshold.
    ///
    /// Correctness rather than cosmetics: `tower::substitution` rolls every
    /// tick, so a sim left running through the sequence advances its RNG stream
    /// by a wall-clock-dependent number of draws and the same seed reaches a
    /// different world (`Stage::world_runs`). How long somebody reads a menu for
    /// is wall-clock time too, and `Threshold` is where the Bevy build spends it.
    pub(crate) const fn clock_runs(&self) -> bool {
        self.boot.is_live() && !self.threshold.is_waiting()
    }

    /// This terminal as a [`Screen`] — no window behind it, so the picture is
    /// the stand-in and hostability turns entirely on the grid.
    pub(super) fn screen(&self) -> Screen {
        Screen::windowless(self.grid, Some(self.held.mode))
    }

    /// Whether this terminal can host the game.
    ///
    /// `Screen::is_hostable`, not a private copy of half of it: `term::fits`
    /// asked only the grid question where the shell asks two — the grid against
    /// the authoring floor *and* the scale — so the frontends routed to the "too
    /// small" card by different rules, invisibly to CI's `--dump` diff.
    pub(super) fn hostable(&self) -> bool {
        self.screen().is_hostable()
    }

    /// The terminal changed size.
    ///
    /// The frame is left for `draw` to reset: blanked here, the next draw would
    /// keep an empty screen for a crossing to depart from. A kept screen of the
    /// old size is refused by `Passing::describes` anyway.
    pub(crate) fn resized(&mut self, cols: u16, rows: u16) {
        self.grid = GridSize::new(cols, rows);
        // The shadow buffer goes with it: the diff is addressed by `(col, row)`,
        // so keeping it across a resize would write this frame's cells at last
        // frame's coordinates.
        self.screen.resize(self.grid);
    }
}
