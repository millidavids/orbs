//! Where in the transcript the player is looking, and whether it has the keys.
//!
//! Pure arithmetic over a record count. The keys that move it are
//! each frontend's own; the word that hands it the keyboard is `unfurl`.

use bevy_ecs::prelude::Resource;
use orbs_sim::Sim;

use crate::screen::Screen;

/// How far back through the transcript the player has scrolled.
///
/// **In records, not rows.** The transcript already finds its tail by
/// binary-searching for the smallest *record* skip whose measured height fits the
/// pane, because a record is not one row — a wrapped message is several, a tiled
/// listing packs many into few, and every command opens with a blank line. Rows
/// would need a second, different measure of the same stream; records reuse the
/// one that is already there and already correct.
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
    /// # Why a verb turns this on
    ///
    /// `PageUp` has scrolled since the transcript existed, and nothing said so:
    /// the border advertises `PgDn newest` only once you are *already* scrolled
    /// back, so the affordance announced itself exclusively to players who had
    /// found it. In a game with no mouse and no menus, that is no affordance at
    /// all — hence `unfurl` (§19), and hence this: the word puts the keys on
    /// screen, which is what the player keeps once they stop needing the word.
    ///
    /// It is the **third** thing that can own the keyboard, after the prompt and
    /// the editor. Which one consumes a keystroke is decided in
    /// the frontend's own key router rather than by a set of run conditions that
    /// had to stay complements — see `orbs`'s `shell::input` for why declining a
    /// key means running.
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
    /// **Escape alone, and it does not scroll anywhere.** Leaving reading mode
    /// is not the same act as returning to the newest output — a player who read
    /// back and pressed Escape wants to type, not to lose their place — so
    /// `PgDn` still walks forward and this only hands the keys over. It is the
    /// same meaning Escape has in the editor: step out of the mode you are in.
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
    /// **`step` is a record count the caller measured**, not a row count. A row
    /// count is not a safe stand-in: a record costs *at least* one row, which
    /// caps records-per-page above rather than below, so paging by the pane's
    /// rows moved more than a screenful and dropped the lines in between. See
    /// `plugin::page_step`, which measures the page with the same `RecordView`
    /// the transcript is drawn with.
    pub fn page(&mut self, step: usize, older: bool, total: usize) {
        let step = step.max(1);
        self.back = if older {
            self.back.saturating_add(step).min(total)
        } else {
            self.back.saturating_sub(step)
        };
    }
}

/// A **conservative** row budget for the transcript body.
///
/// The pane's own body is shorter than the grid once its border, the prompt, a
/// Tab listing and §10.1's instrument panel are taken out, and `paint` is the
/// only thing that knows exactly. Under-estimating is the safe direction: a page
/// that moves slightly less than a screenful overlaps the last one by a line or
/// two, which is what a reader wants anyway.
fn page_rows(screen: &Screen) -> u16 {
    screen.grid.rows.saturating_sub(6).max(1)
}

/// How many records the transcript is currently showing.
///
/// **Measured, not assumed.** [`Scroll`] moves in records, and the step used to
/// be the pane's *row* count on the reasoning that a record costs at least one
/// row, so a page can never hold more records than rows. That bound runs the
/// other way: "at least a row each" caps records-per-page *above*, so stepping
/// by the row count moves further than a screenful and `PgUp` jumped clean over
/// the lines in between — drawn at neither end of the jump. It is now wrong by
/// more than it was, because `RecordView` opens every command with a blank row
/// and wraps a long line over several.
///
/// So this runs the same monotone search `prompt::session` draws with: the
/// smallest skip whose measured height fits. The step is then the page the
/// player is actually looking at, and paging can only ever overlap.
///
/// **Shared rather than reimplemented per frontend**, because it is not a
/// keyboard question — it is *what the transcript would fit*, measured with the
/// same [`RecordView`](orbs_render::RecordView) the transcript is drawn with. A
/// second measure of the same stream would page by a different amount than it
/// showed.
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
