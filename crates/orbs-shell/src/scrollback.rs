//! Where in the transcript the player is looking, and whether it has the keys.
//!
//! Pure arithmetic over a record count. The keys that move it are
//! each frontend's own; the word that hands it the keyboard is `unfurl`.

use bevy_ecs::prelude::Resource;
use orbs_sim::Sim;

use crate::screen::Screen;

/// How far back through the transcript the player has scrolled.
///
/// In records, not rows: a record is not one row — a wrapped message is
/// several, a tiled listing packs many into few — and the transcript already
/// finds its tail by binary-searching record skips. Rows would need a second
/// measure of the same stream.
///
/// Zero is the bottom, which is where it returns on every submission: you typed
/// something, so you want to see what it did.
#[derive(Resource, Debug, Default)]
pub struct Scroll {
    /// Records held back from the newest end.
    back: usize,
    /// Whether the transcript has the keyboard — see [`Scroll::is_reading`].
    reading: bool,
}

impl Scroll {
    /// How far back the view is.
    #[must_use]
    pub const fn back(&self) -> usize {
        self.back
    }

    /// Whether the transcript currently has the keyboard.
    ///
    /// `PageUp` has always scrolled, but the border advertises `PgDn newest`
    /// only once you are *already* scrolled back — an affordance that announced
    /// itself only to players who had found it. Hence `unfurl` (§19): the word
    /// puts the keys on screen.
    ///
    /// The third thing that can own the keyboard, after the prompt and the
    /// editor. Which one consumes a keystroke is decided in the frontend's own
    /// key router — see `orbs`'s `shell::input`.
    #[must_use]
    pub const fn is_reading(&self) -> bool {
        self.reading
    }

    /// Take the keyboard, and start from where the view already is.
    pub const fn read(&mut self) {
        self.reading = true;
    }

    /// Give the keyboard back to the prompt.
    ///
    /// Escape alone, and it does not scroll: leaving reading mode is not the
    /// same act as returning to the newest output — a player who read back and
    /// pressed Escape wants to type, not to lose their place. Same meaning
    /// Escape has in the editor.
    pub const fn stop_reading(&mut self) {
        self.reading = false;
    }

    /// Whether the player is looking at history rather than at the newest output.
    #[must_use]
    pub const fn is_back(&self) -> bool {
        self.back > 0
    }

    /// Return to the newest output.
    pub const fn rewind(&mut self) {
        self.back = 0;
    }

    /// Move by `step` records, older or newer. Clamped at both ends.
    ///
    /// `step` is a record count the caller measured, not a row count: a record
    /// costs *at least* one row, which caps records-per-page above rather than
    /// below, so paging by the pane's rows moved more than a screenful and
    /// dropped the lines in between. See `plugin::page_step`.
    pub fn page(&mut self, step: usize, older: bool, total: usize) {
        let step = step.max(1);
        self.back = if older {
            self.back.saturating_add(step).min(total)
        } else {
            self.back.saturating_sub(step)
        };
    }
}

/// A conservative row budget for the transcript body.
///
/// Only `paint` knows exactly how short the body is once the border, the
/// prompt, a Tab listing and §10.1's instrument panel are taken out.
/// Under-estimating is the safe direction: a page that moves slightly less than
/// a screenful overlaps the last one by a line or two.
fn page_rows(screen: &Screen) -> u16 {
    screen.grid.rows.saturating_sub(6).max(1)
}

/// How many records the transcript is currently showing.
///
/// Measured, not assumed. The step used to be the pane's *row* count, on the
/// reasoning that a record costs at least one row — but that bound caps
/// records-per-page *above*, so stepping by rows moved further than a screenful
/// and `PgUp` jumped clean over the lines in between.
///
/// So this runs the same monotone search `prompt::session` draws with: the
/// smallest skip whose measured height fits, which can only ever overlap.
///
/// Shared rather than reimplemented per frontend: it is *what the transcript
/// would fit*, measured with the same
/// [`RecordView`](orbs_render::RecordView) it is drawn with.
#[must_use]
pub fn page_step(screen: &Screen, sim: &Sim, back: usize) -> usize {
    let records = sim.scrollback().records();
    let visible = records.drawn_len().saturating_sub(back);
    if visible == 0 {
        return 1;
    }
    let rows = page_rows(screen);
    let cols = screen.grid.cols.saturating_sub(2);
    let prompt = sim.prompt();
    let view = orbs_render::RecordView::prompt(&prompt);
    let (mut narrowest, mut widest) = (0, visible);
    while narrowest < widest {
        let candidate = narrowest + (widest - narrowest) / 2;
        if view.height(cols, records.drawn().take(visible).skip(candidate)) <= rows {
            widest = candidate;
        } else {
            narrowest = candidate + 1;
        }
    }
    (visible - narrowest).max(1)
}
